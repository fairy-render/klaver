use core::fmt;

use rquickjs::{CaughtError, Coerced};

use crate::error::StackTrace;

#[derive(Debug, Clone)]
pub struct CaugthException {
    pub message: Option<String>,
    pub stack: Vec<StackTrace>,
}

impl fmt::Display for CaugthException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                        Err(_err) => Vec::default(),
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
            CaughtError::Value(e) => {
                //
                if let Some(object) = e.as_object() {
                    let message = if let Ok(message) = object.get::<_, String>("message") {
                        Some(message)
                    } else {
                        None
                    };
                    if let Ok(stack) = object.get::<_, String>("stack") {
                        let traces = match super::stack_trace::parse(&stack) {
                            Ok(ret) => ret,
                            Err(_err) => Vec::default(),
                        };
                        return CaugthException {
                            message: message,
                            stack: traces,
                        };
                    }
                }
                CaugthException {
                    message: e.get::<Coerced<String>>().map(|m| m.0).ok(),
                    stack: Default::default(),
                }
            }
        }
    }
}
