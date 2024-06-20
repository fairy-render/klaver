use core::fmt;
use rquickjs::{module::Evaluated, AsyncContext, AsyncRuntime, Ctx, Module, Persistent, Value};
use slotmap::SlotMap;
use std::{any::Any, cell::RefCell, future::Future, pin::Pin, rc::Rc};
use tokio::{
    sync::{mpsc, oneshot},
    task::LocalSet,
};

pub type WithAsyncFn = Box<
    dyn for<'a> Fn(
            Ctx<'a>,
            Persistence,
        ) -> Pin<
            Box<dyn Future<Output = Result<Box<dyn Any + Send>, rquickjs::Error>> + 'a + Send>,
        > + Send,
>;

#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "futures")))]
#[macro_export]
macro_rules! async_with{
  ($context:expr => |$ctx:ident, $persistence: ident| { $($t:tt)* }) => {
      $crate::Worker::with_async(&$context, move|$ctx, $persistence| {
          let fut = Box::pin(async move {
              $($t)*
          });
          /// SAFETY: While rquickjs objects have a 'js lifetime attached to them,
          /// they actually life much longer an the lifetime is just for checking
          /// if they belong to the correct context.
          /// By requiring that everything is moved into the closure outside
          /// environments still can't life shorter than the closure.
          /// This allows use to recast the future to a higher lifetime without problems.
          /// Second, the future will always acquire a lock before running. The closure
          /// enforces that everything moved into the future is send, but non of the
          /// rquickjs objects are send so the future will never be send.
          /// Since we acquire a lock before running the future and nothing can escape the closure
          /// and future it is safe to recast the future as send.
          unsafe fn uplift<'a,'b,R>(f: std::pin::Pin<Box<dyn std::future::Future<Output = R> + 'a>>) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + 'b + Send>>{
              std::mem::transmute(f)
          }
          unsafe{ uplift(fut) }
      })
  };
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct RefId {
//     id: DefaultKey,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct ModuleId {
//     id: DefaultKey,
// }

slotmap::new_key_type! {
  pub struct ModuleId;
  pub struct RefId;
}

#[derive(Debug, Clone)]
pub struct Persistence {
    values: Rc<RefCell<SlotMap<RefId, Persistent<Value<'static>>>>>,
    modules: Rc<RefCell<SlotMap<ModuleId, Persistent<Module<'static, Evaluated>>>>>,
}

unsafe impl Send for Persistence {}

unsafe impl Sync for Persistence {}

impl Persistence {
    fn new() -> Self {
        Persistence {
            values: Default::default(),
            modules: Default::default(),
        }
    }
}

impl Persistence {
    pub fn save<'js>(&self, ctx: &Ctx<'js>, val: Value<'js>) -> rquickjs::Result<RefId> {
        let v = Persistent::save(ctx, val);
        let id = self.values.borrow_mut().insert(v);
        Ok(id)
    }

    pub fn save_module<'js>(
        &self,
        ctx: &Ctx<'js>,
        val: Module<'js, Evaluated>,
    ) -> rquickjs::Result<ModuleId> {
        let v = Persistent::save(ctx, val);
        let id = self.modules.borrow_mut().insert(v);
        Ok(id)
    }

    pub fn load<'js>(&self, ctx: &Ctx<'js>, key: &RefId) -> rquickjs::Result<Value<'js>> {
        self.values.borrow()[*key].clone().restore(ctx)
    }

    pub fn load_module<'js>(
        &self,
        ctx: &Ctx<'js>,
        key: &ModuleId,
    ) -> rquickjs::Result<Module<'js, Evaluated>> {
        self.modules.borrow()[*key].clone().restore(ctx)
    }

    pub fn close(&self, id: RefId) {
        self.values.borrow_mut().remove(id);
    }

    pub fn close_module(&self, id: ModuleId) {
        self.modules.borrow_mut().remove(id);
    }
}

#[derive(Debug)]
pub enum Error {
    Script(rquickjs::Error),
    Channel,
    Downcast,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Channel => write!(f, "channel closed"),
            Self::Downcast => write!(f, "downcast failed"),
            Self::Script(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<rquickjs::Error> for Error {
    fn from(value: rquickjs::Error) -> Self {
        Error::Script(value)
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(_value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Error::Channel
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(_value: tokio::sync::oneshot::error::RecvError) -> Self {
        Error::Channel
    }
}

pub struct Worker {
    sx: mpsc::Sender<Request>,
}

impl Worker {
    pub async fn new() -> Result<Worker, Error> {
        let sx = create_worker().await?;
        Ok(Worker { sx })
    }

    pub async fn with<T, R>(&self, func: T) -> Result<R, Error>
    where
        T: Send + 'static,
        for<'js> T: FnOnce(Ctx<'js>) -> rquickjs::Result<R>,
        R: Send + 'static,
    {
        let (sx, rx) = oneshot::channel();

        self.sx
            .send(Request::With {
                func: Box::new(move |ctx| {
                    let ret = func(ctx)?;
                    Ok(Box::new(ret))
                }),
                returns: sx,
            })
            .await?;

        let ret = rx.await??;

        Ok(*ret.downcast().map_err(|_| Error::Downcast)?)
    }

    pub async fn with_async<T, R>(&self, func: T) -> Result<R, Error>
    where
        T: Send,
        for<'js> T: FnOnce(
                Ctx<'js>,
                Persistence,
            )
                -> Pin<Box<dyn Future<Output = Result<R, rquickjs::Error>> + 'js + Send>>
            + 'js,
        R: Send + 'static,
    {
        let (sx, rx) = oneshot::channel();

        self.sx
            .send(Request::WithAsync {
                func: Box::new(move |ctx, p| {
                    Box::pin(async {
                        let future = func(ctx, p);
                        let ret = future.await?;
                        Ok(Box::new(ret) as Box<dyn Any + Send>)
                    })
                }),
                returns: sx,
            })
            .await?;

        let ret = rx.await??;

        Ok(*ret.downcast().map_err(|_| Error::Downcast)?)
    }

    pub async fn customize<T, R>(&self, func: T) -> Result<(), Error>
    where
        T: Send + 'static,
        for<'js> T: FnOnce(
            &'js AsyncRuntime,
            &'js AsyncContext,
        )
            -> Pin<Box<dyn Future<Output = Result<R, rquickjs::Error>> + 'js>>,
        R: Send + 'static,
    {
        let (sx, rx) = oneshot::channel();

        self.sx
            .send(Request::Customize {
                func: Box::new(move |runtime, ctx| {
                    Box::pin(async {
                        let future = func(runtime, ctx);
                        let ret = future.await?;
                        Ok(Box::new(ret) as Box<dyn Any + Send>)
                    })
                }),
                returns: sx,
            })
            .await?;

        let ret = rx.await??;

        Ok(*ret.downcast().map_err(|_| Error::Downcast)?)
    }

    pub async fn close(&self, id: RefId) {
        self.sx.send(Request::Close { id }).await.ok();
    }
}

enum Request {
    With {
        func:
            Box<dyn for<'a> FnOnce(Ctx<'a>) -> Result<Box<dyn Any + Send>, rquickjs::Error> + Send>,
        returns: oneshot::Sender<Result<Box<dyn Any + Send>, rquickjs::Error>>,
    },
    WithAsync {
        func: Box<
            dyn for<'a> FnOnce(
                    Ctx<'a>,
                    Persistence,
                ) -> Pin<
                    Box<
                        dyn Future<Output = Result<Box<dyn Any + Send>, rquickjs::Error>>
                            + 'a
                            + Send,
                    >,
                > + Send,
        >,
        returns: oneshot::Sender<Result<Box<dyn Any + Send>, rquickjs::Error>>,
    },

    Customize {
        func: Box<
            dyn for<'a> FnOnce(
                    &'a AsyncRuntime,
                    &'a AsyncContext,
                ) -> Pin<
                    Box<dyn Future<Output = Result<Box<dyn Any + Send>, rquickjs::Error>> + 'a>,
                > + Send,
        >,
        returns: oneshot::Sender<Result<Box<dyn Any + Send>, rquickjs::Error>>,
    },
    Close {
        id: RefId,
    },
}

async fn create_worker() -> Result<mpsc::Sender<Request>, Error> {
    let (sx, mut rx) = mpsc::channel(1);
    let (ready, wait) = oneshot::channel::<Option<Error>>();

    let weak_sx = sx.downgrade();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(ret) => ret,
            Err(err) => {
                ready.send(Some(rquickjs::Error::Io(err).into())).ok();
                return;
            }
        };

        let local_set = LocalSet::new();

        let runtime = match AsyncRuntime::new() {
            Ok(ret) => ret,
            Err(err) => {
                ready.send(Some(err.into())).ok();
                return;
            }
        };

        local_set.spawn_local(async move {
            let ctx = match AsyncContext::full(&runtime).await {
                Ok(ret) => ret,
                Err(err) => {
                    ready.send(Some(err.into())).ok();
                    return;
                }
            };

            if ready.send(None).is_err() {
                return;
            }

            tokio::task::spawn_local(runtime.drive());

            let persistence = Persistence::new();

            while let Some(next) = rx.recv().await {
                match next {
                    Request::With { func, returns } => {
                        let ret = ctx.with(func).await;
                        returns.send(ret).ok();
                    }
                    Request::WithAsync { func, returns } => {
                        // let persistence = persistence.clone();

                        let persistence = persistence.clone();

                        let ret = ctx.async_with(move |ctx| func(ctx, persistence)).await;
                        // let ret = rquickjs::async_with!(ctx => |ctx| {
                        //     func(ctx, persistence).await
                        // })
                        // .await;
                        returns.send(ret).ok();
                    }
                    Request::Customize { func, returns } => {
                        let ret = func(&runtime, &ctx).await;
                        returns.send(ret).ok();
                    }
                    Request::Close { id } => {
                        persistence.close(id);
                    }
                }
            }

            drop(persistence);
        });

        rt.block_on(local_set)
    });

    if let Some(err) = wait.await? {
        return Err(err);
    }

    Ok(sx)
}
