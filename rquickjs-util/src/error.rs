use rquickjs::CaughtError;

use crate::stack_trace::{self, StackTrace};

#[derive(Debug)]
pub enum RuntimeError {
    Quick(rquickjs::Error),
    Custom(Box<dyn std::error::Error + Send + Sync>),
    Message(Option<String>),
    Exception {
        message: Option<String>,
        stack: Vec<StackTrace>,
    },
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::Message(msg) => write!(f, "{:?}", *msg),
            RuntimeError::Quick(e) => write!(f, "{e}"),
            RuntimeError::Custom(e) => write!(f, "{e}"),
            RuntimeError::Exception { message, stack } => {
                //
                "Error:".fmt(f)?;

                if let Some(message) = message {
                    ' '.fmt(f)?;
                    message.fmt(f)?;
                }
                for trace in stack {
                    write!(f, "\n  at {trace}")?;
                }

                Ok(())
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

impl<'js> From<CaughtError<'js>> for RuntimeError {
    fn from(value: CaughtError<'js>) -> Self {
        match value {
            CaughtError::Error(err) => err.into(),
            CaughtError::Exception(e) => {
                let stack = if let Some(stack) = e.stack() {
                    let traces = match stack_trace::parse(&stack) {
                        Ok(ret) => ret,
                        Err(_err) => Vec::default(),
                    };
                    traces
                } else {
                    Vec::default()
                };
                RuntimeError::Exception {
                    message: e.message(),
                    stack: stack,
                }
            }
            CaughtError::Value(e) => {
                RuntimeError::Message(e.as_string().and_then(|m| m.to_string().ok()))
            }
        }
    }
}

impl From<rquickjs::Error> for RuntimeError {
    fn from(value: rquickjs::Error) -> Self {
        RuntimeError::Quick(value)
    }
}
