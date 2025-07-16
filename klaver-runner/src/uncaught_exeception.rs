use rquickjs::CaughtError;
use rquickjs_util::{RuntimeError, StackTrace};

#[derive(Debug, Clone)]
pub struct UncaugthException {
    message: Option<String>,
    stack: Vec<StackTrace>,
}

impl<'js> From<CaughtError<'js>> for UncaugthException {
    fn from(value: CaughtError<'js>) -> Self {
        match value {
            CaughtError::Error(err) => UncaugthException {
                message: Some(err.to_string()),
                stack: Default::default(),
            },
            CaughtError::Exception(e) => {
                let stack = if let Some(stack) = e.stack() {
                    let traces = match rquickjs_util::stack_trace::parse(&stack) {
                        Ok(ret) => ret,
                        Err(_err) => Vec::default(),
                    };
                    traces
                } else {
                    Vec::default()
                };
                UncaugthException {
                    message: e.message(),
                    stack,
                }
            }
            CaughtError::Value(e) => UncaugthException {
                message: e.as_string().and_then(|m| m.to_string().ok()),
                stack: Default::default(),
            },
        }
    }
}

impl From<UncaugthException> for RuntimeError {
    fn from(value: UncaugthException) -> Self {
        RuntimeError::Exception {
            message: value.message,
            stack: value.stack,
        }
    }
}
