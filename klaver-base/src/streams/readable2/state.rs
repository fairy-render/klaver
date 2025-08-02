use klaver_util::sync::ObservableCloneCell;
use rquickjs::{JsLifetime, String, Value, class::Trace};

use super::queue::Queue;

#[derive(Trace, Debug, Clone)]
pub enum StreamState<'js> {
    Aborted(Option<String<'js>>),
    Failed(Value<'js>),
    Closed,
    Running,
    Locked,
    Done,
}

#[derive(Trace)]
#[rquickjs::class]
pub struct ReadableStreamData<'js> {
    pub queue: Queue<'js>,
    pub state: ObservableCloneCell<StreamState<'js>>,
}

unsafe impl<'js> JsLifetime<'js> for ReadableStreamData<'js> {
    type Changed<'to> = ReadableStreamData<'to>;
}
