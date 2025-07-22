use std::time::Duration;

use super::TimeId;
use futures::channel::oneshot;
use rquickjs::Function;

pub struct Timer<'js> {
    pub id: TimeId,
    pub callback: Function<'js>,
    pub repeat: bool,
    pub duration: Duration,
}

impl<'js> Timer<'js> {
    pub async fn run(mut self, mut kill: oneshot::Receiver<()>) -> rquickjs::Result<()> {
        if self.repeat {
            loop {
                let ret = self.run_inner(&mut kill).await?;
                if !ret {
                    break;
                }
            }
        } else {
            self.run_inner(&mut kill).await?;
        }

        Ok(())
    }

    async fn run_inner(&mut self, kill: &mut oneshot::Receiver<()>) -> rquickjs::Result<bool> {
        let timeout = tokio::time::sleep(self.duration);

        tokio::select! {
            _ = timeout => {
                self.callback.call::<_,()>(())?;
                Ok(true)
            }
            _ = kill => {
                Ok(false)
            }
        }
    }
}
