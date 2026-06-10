use klaver_core::value::structured_clone::TransferData;
use rquickjs::{Ctx, Value};

pub enum Message {
    Kill,
    Event(TransferData),
}

fn post_message<'js>(_ctx: Ctx<'js>, _msg: Value<'js>) -> rquickjs::Result<()> {
    Ok(())
}

// pub fn work(path: &str, registry: Registry, channel: Channel) -> Result<(), RuntimeError> {
//     futures::executor::block_on(async move {
//         let runtime = AsyncRuntime::new()?;
//         let context = AsyncContext::full(&runtime).await?;

//         context
//             .with(move |ctx| {
//                 ctx.store_userdata(registry.clone())?;

//                 let port = MessagePort::new(ctx.clone())?;

//                 ctx.globals().set("port", port)
//             })
//             .await?;

//         Runner::new(
//             &context,
//             Work {
//                 path: path.to_string(),
//             },
//         )
//         .run()
//         .await?;

//         Ok(())
//     })
// }

// struct Work {
//     path: std::string::String,
// }

// impl Runnerable for Work {
//     type Future<'js> = LocalBoxFuture<'js, Result<(), RuntimeError>>;

//     fn call<'js>(self, ctx: Ctx<'js>, worker: klaver_runner::Workers) -> Self::Future<'js> {
//         Box::pin(async move {
//             // worker.push(ctx.clone(), |ctx, mut shutdown| async move {
//             //     //

//             //     let trigger = ctx
//             //         .globals()
//             //         .get::<_, Function>("__triggerMessage")
//             //         .catch(&ctx)?;

//             //     loop {
//             //         futures::select! {
//             //             _ = shutdown => {
//             //                 break
//             //             }
//             //             val = self.rx.recv_async() => {
//             //                 let Ok(val) = val else {
//             //                     break;
//             //                 };

//             //                 match val {
//             //                     Message::Event(val) => {
//             //                         trigger.call::<_, ()>((val,)).catch(&ctx)?;
//             //                     }
//             //                     Message::Kill => {
//             //                         break
//             //                     }
//             //                 }
//             //             }
//             //         }
//             //     }

//             //     Ok(())
//             // });

//             Module::import(&ctx, self.path)
//                 .catch(&ctx)?
//                 .into_future::<()>()
//                 .await?;

//             Ok(())
//         })
//     }
// }
