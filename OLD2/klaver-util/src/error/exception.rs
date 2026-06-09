use core::fmt;

use rquickjs::CaughtError;

use crate::error::StackTrace;

#[derive(Debug, Clone)]
pub struct CaugthException {
    pub message: Option<String>,
    pub stack: Vec<StackTrace>,
}

impl fmt::Display for CaugthException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "Error:".fmt(f)?;

        if let Some(message) = &self.message {
            ' '.fmt(f)?;
            message.fmt(f)?;
        }
        for trace in &self.stack {
            write!(f, "\n  at {trace}")?;
        }
        Ok(())
    }
}

impl std::error::Error for CaugthException {}

impl<'js> From<CaughtError<'js>> for CaugthException {
    fn from(value: CaughtError<'js>) -> Self {
        match value {
            CaughtError::Error(err) => CaugthException {
                message: Some(err.to_string()),
                stack: Default::default(),
            },
            CaughtError::Exception(e) => {
                let stack = if let Some(stack) = e.stack() {
                    let traces = match super::stack_trace::parse(&stack) {
                        Ok(ret) => ret,
                        Err(_err) => {
                            println!("ERROR {}", _err);
                            Vec::default()
                        }
                    };
                    traces
                } else {
                    Vec::default()
                };

                CaugthException {
                    message: e.message(),
                    stack,
                }
            }
            CaughtError::Value(e) => CaugthException {
                message: e.as_string().and_then(|m| m.to_string().ok()),
                stack: Default::default(),
            },
        }
    }
}
