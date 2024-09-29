use klaver::throw_if;
use klaver_shared::{iter::AsyncIter, Static};
use rquickjs::{class::Trace, Ctx, IntoJs, Value};
use tokio::io::AsyncBufReadExt;

#[rquickjs::class]
#[derive(Trace)]
pub struct JsFile {
    pub file: Static<tokio::fs::File>,
}

impl JsFile {
    pub fn new(file: tokio::fs::File) -> JsFile {
        JsFile { file: Static(file) }
    }
}

#[rquickjs::methods]
impl JsFile {
    #[qjs(rename = "readLines")]
    pub async fn read_lines<'js>(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        let file = throw_if!(ctx, self.file.try_clone().await);
        let lines = tokio::io::BufReader::new(file).lines();

        let stream = tokio_stream::wrappers::LinesStream::new(lines);

        AsyncIter::new(Static(stream)).into_js(&ctx)
    }
}
