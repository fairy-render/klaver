use klaver_util::rquickjs::{self, AsyncContext};

pub struct EventLoop {}

impl EventLoop {
    pub fn run(self, context: &AsyncContext) {
        rquickjs::async_with!(context => |ctx| {

        });
    }
}
