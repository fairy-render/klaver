use rquickjs::{JsLifetime, String, class::Trace};

use crate::blob::Blob;

#[derive(Debug, JsLifetime)]
pub struct File<'js> {
    blob: Blob<'js>,
    file_name: String<'js>,
}
