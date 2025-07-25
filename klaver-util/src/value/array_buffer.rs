use rquickjs::Ctx;

pub struct ArrayBuffer {
    data: Vec<u8>,
    max_size: usize,
}

impl ArrayBuffer {
    pub fn resize<'js>(&mut self, ctx: Ctx<'js>, size: usize) -> rquickjs::Result<()> {
        Ok(())
    }
}
