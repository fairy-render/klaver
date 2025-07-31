use klaver_task::{Resource, ResourceId};
use rquickjs::Class;

use crate::streams::readable2::state::ReadableStreamData;

pub struct ReadableStreamResourceId;

impl ResourceId for ReadableStreamResourceId {
    fn name() -> &'static str {
        "ReadableStreamWrap"
    }
}

pub struct ReadableStreamResource<'js> {
    data: Class<'js, ReadableStreamData<'js>>,
}

impl<'js> Resource<'js> for ReadableStreamResource<'js> {
    type Id = ReadableStreamResourceId;
    const INTERNAL: bool = true;
    const SCOPED: bool = true;

    async fn run(self, ctx: klaver_task::TaskCtx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}
