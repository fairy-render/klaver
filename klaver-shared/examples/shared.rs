use klaver_shared::date::Date;
use rquickjs::{CatchResultExt, Context, Runtime};

fn main() {
    let runtime = Runtime::new().unwrap();
    let context = Context::full(&runtime).unwrap();

    context
        .with(|ctx| {
            let date = ctx.eval::<Date, _>("new Date")?;

            println!("Date {}", date.year()?);

            // let date = Date::from_chrono(&ctx, chrono::Utc::now())?;

            println!("Date {}", date.to_string().catch(&ctx).unwrap());

            rquickjs::Result::Ok(())
        })
        .unwrap();
}
