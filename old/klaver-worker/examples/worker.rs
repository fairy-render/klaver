use klaver_worker::pool::{Manager, Pool};
use rquickjs::function::Func;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main_wrapped())
}

async fn main_wrapped() {
    let pool = Pool::builder(Manager::new_with(|ctx, p| {
        Box::pin(async move {
            ctx.globals().set(
                "add",
                Func::new(|a: i32, b: i32| rquickjs::Result::Ok(a + b)),
            )
        })
    }))
    .build()
    .unwrap();

    let worker = pool.get().await.unwrap();

    let val = klaver_worker::async_with!(*worker => |ctx, p| {
      let val: i32 = ctx.eval("add(2, 2)")?;
      Ok(val)
    })
    .await
    .unwrap();

    println!("val {}", val);
}
