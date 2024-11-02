use rquickjs::{class::Trace};
use tokio::sync::oneshot;

#[rquickjs::class]
pub struct Cancel {
    sx: Option<oneshot::Sender<()>>,
}

impl<'js> Trace<'js> for Cancel {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl Cancel {
    pub fn create(&mut self) -> Option<oneshot::Receiver<()>> {
        if self.sx.is_some() {
            return None;
        }
        let (sx, rx) = oneshot::channel();
        self.sx = Some(sx);

        Some(rx)
    }
}

#[rquickjs::methods]
impl Cancel {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Cancel { sx: None }
    }

    pub fn cancel(&mut self) {
        let Some(sx) = self.sx.take() else { return };
        sx.send(()).ok();
    }
}
