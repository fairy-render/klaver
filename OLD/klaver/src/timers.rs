use rquickjs::AsyncContext;

use crate::{
    base::timers::{poll_timers, process_timers},
    Error,
};

pub async fn wait_timers<'a>(context: &'a AsyncContext) -> Result<(), Error> {
    loop {
        let has_timers = context.with(|ctx| process_timers(&ctx)).await?;

        if !has_timers && !context.runtime().is_job_pending().await {
            break;
        }

        let sleep = context.with(|ctx| poll_timers(&ctx)).await?;

        sleep.await;
    }

    Ok(())
}
