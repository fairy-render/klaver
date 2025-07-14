use crate::blob::{Blob, BlobInit};
use rquickjs::{class::Trace, Ctx, JsLifetime, String};
use rquickjs_util::StringRef;

#[rquickjs::class]
#[derive(Trace)]
pub struct File<'js> {
    blob: Blob<'js>,
    #[qjs(get)]
    file_name: String<'js>,
}

unsafe impl<'js> JsLifetime<'js> for File<'js> {
    type Changed<'to> = File<'to>;
}

impl<'js> File<'js> {
    pub fn init(ctx: Ctx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}

impl<'js> File<'js> {
    fn new(init: Vec<BlobInit<'js>>, file_name: String<'js>) -> rquickjs::Result<File<'js>> {
        let blob = Blob::new(init)?;

        Ok(File { blob, file_name })
    }
}
