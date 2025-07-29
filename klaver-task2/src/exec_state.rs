use core::fmt;
use event_listener::{Event, EventListener};
use klaver_util::rquickjs::{self, IntoJs, Value, class::Trace};
use std::{cell::RefCell, collections::HashMap, rc::Rc, usize};

use crate::cell::ObservableCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsyncId(usize);

impl AsyncId {
    pub fn root() -> AsyncId {
        AsyncId(0)
    }
}

impl<'js> Trace<'js> for AsyncId {
    fn trace<'a>(&self, _tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for AsyncId {
    fn into_js(
        self,
        ctx: &klaver_util::rquickjs::Ctx<'js>,
    ) -> klaver_util::rquickjs::Result<klaver_util::rquickjs::Value<'js>> {
        Ok(Value::new_int(ctx.clone(), self.0 as _))
    }
}

pub struct Task {
    parent: Option<AsyncId>,
    children: usize,
    shutdown: Rc<ObservableCell<bool>>,
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("shutdown", &self.shutdown.get())
            .finish()
    }
}

#[derive(Debug)]
struct Inner {
    next_id: usize,
    current_id: Vec<AsyncId>,
    execution_id: Vec<AsyncId>,
    tasks: HashMap<AsyncId, Task>,
    event: Rc<Event>,
}

impl Inner {
    fn notify_shutdown(&self, parent: AsyncId) -> rquickjs::Result<()> {
        let Some(task) = self.tasks.get(&parent) else {
            return Ok(());
        };

        for (id, task) in &self.tasks {
            if task.parent == Some(parent) {
                self.notify_shutdown(*id)?;
            }
        }
        task.shutdown.set(true);

        Ok(())
    }
}

#[derive(Clone)]
pub struct ExecState(Rc<RefCell<Inner>>);

impl Default for ExecState {
    fn default() -> Self {
        ExecState(Rc::new(RefCell::new(Inner {
            // next_id: 1,
            // current_id: vec![AsyncId(0)],
            // execution_id: vec![AsyncId(0)],
            // tasks,
            tasks: Default::default(),
            next_id: 0,
            current_id: Default::default(),
            execution_id: Default::default(),
            event: Rc::new(Event::new()),
        })))
    }
}

impl ExecState {
    pub fn dump(&self) {
        println!("{:#?}", &*self.0.borrow());
    }

    pub fn set_current(&self, current: AsyncId) {
        self.0.borrow_mut().current_id.push(current);
    }

    pub fn enter<T, R>(&self, id: AsyncId, func: T) -> R
    where
        T: FnOnce() -> R,
    {
        {
            let mut inner = self.0.borrow_mut();
            if let Some(task) = inner.tasks.get(&id) {
                if let Some(parent) = task.parent {
                    inner.execution_id.push(parent);
                } else {
                    inner.execution_id.push(AsyncId::root());
                }
            } else if let Some(current) = inner.current_id.last().copied() {
                inner.execution_id.push(current);
            } else {
                inner.execution_id.push(AsyncId::root());
            }
            inner.current_id.push(id);
        }
        let ret = func();
        self.0.borrow_mut().current_id.pop();
        self.0.borrow_mut().execution_id.pop();
        ret
    }

    pub async fn enter_async<T, U, R>(&self, id: AsyncId, func: T) -> R
    where
        T: FnOnce() -> U,
        U: Future<Output = R>,
    {
        {
            let mut inner = self.0.borrow_mut();
            if let Some(current) = inner.current_id.last().copied() {
                inner.execution_id.push(current);
            } else if let Some(task) = inner.tasks.get(&id) {
                if let Some(parent) = task.parent {
                    inner.execution_id.push(parent);
                } else {
                    inner.execution_id.push(AsyncId::root());
                }
            } else {
                inner.execution_id.push(AsyncId::root());
            }
            inner.current_id.push(id);
        }

        let ret = func().await;

        self.0.borrow_mut().current_id.pop();
        self.0.borrow_mut().execution_id.pop();
        ret
    }

    pub fn listen(&self) -> EventListener {
        self.0.borrow().event.listen()
    }

    pub async fn shutdown(&self, id: AsyncId) -> rquickjs::Result<()> {
        if self.child_count(id) == 0 {
            return Ok(());
        }

        self.0.borrow().notify_shutdown(id)?;

        loop {
            if self.child_count(id) == 0 {
                break;
            }

            let listener = self.0.borrow().event.listen();

            listener.await;
        }

        Ok(())
    }

    // Wait until the task has shutdown status
    pub async fn wait_shutdown(&self, id: AsyncId) -> rquickjs::Result<()> {
        let cell = if let Some(task) = self.0.borrow().tasks.get(&id) {
            task.shutdown.clone()
        } else {
            return Ok(());
        };

        loop {
            if cell.get() {
                break;
            }

            cell.subscribe().await;
        }

        Ok(())
    }

    pub async fn wait_children(&self, id: AsyncId) {
        loop {
            if self.child_count(id) == 0 {
                break;
            }

            let listener = self.0.borrow().event.listen();

            listener.await;
        }
    }

    pub fn create_task(&self, parent: Option<AsyncId>) -> AsyncId {
        let id = self.0.borrow().next_id;
        self.0.borrow_mut().next_id += 1;
        let id = AsyncId(id);

        let shutdown = if let Some(parent) = parent {
            let shutdown = if let Some(parent) = self.0.borrow().tasks.get(&parent) {
                parent.shutdown.get()
            } else {
                false
            };

            self.0.borrow_mut().tasks.get_mut(&parent).unwrap().children += 1;
            shutdown
        } else {
            false
        };

        self.0.borrow_mut().tasks.insert(
            id,
            Task {
                parent: parent,
                children: 0,
                shutdown: Rc::new(ObservableCell::new(shutdown)),
            },
        );

        self.0.borrow().event.notify(usize::MAX);

        id
    }

    pub fn child_count(&self, id: AsyncId) -> usize {
        self.0
            .borrow()
            .tasks
            .get(&id)
            .map(|m| m.children)
            .unwrap_or_default()
    }

    pub fn destroy_task(&self, id: AsyncId) {
        let Some(task) = self.0.borrow_mut().tasks.remove(&id) else {
            return;
        };

        if let Some(parent) = task.parent {
            if let Some(parent) = self.0.borrow_mut().tasks.get_mut(&parent) {
                parent.children -= 1;
            }
        }

        self.0.borrow().event.notify(usize::MAX);
    }

    pub fn trigger_async_id(&self) -> Option<AsyncId> {
        self.0.borrow().current_id.last().copied()
    }

    pub fn exectution_trigger_id(&self) -> Option<AsyncId> {
        self.0.borrow().execution_id.last().copied()
    }
}
