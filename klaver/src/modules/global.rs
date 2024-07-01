use rquickjs::Ctx;

pub trait Global {
    fn init<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<()>;
}
