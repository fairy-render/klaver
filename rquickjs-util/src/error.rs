use rquickjs::CaughtError;

#[derive(Debug)]
pub enum RuntimeError {
    Quick(rquickjs::Error),
    Custom(Box<dyn std::error::Error + Send + Sync>),
    Message(Option<String>),
    Exception {
        line: Option<i32>,
        column: Option<i32>,
        message: Option<String>,
        stack: Option<String>,
        file: Option<String>,
    },
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::Message(msg) => write!(f, "{:?}", *msg),
            RuntimeError::Quick(e) => write!(f, "{e}"),
            RuntimeError::Custom(e) => write!(f, "{e}"),
            RuntimeError::Exception {
                line,
                column,
                message,
                stack,
                file,
            } => {
                //
                "Error:".fmt(f)?;
                let mut has_file = false;
                if let Some(file) = file {
                    '['.fmt(f)?;
                    file.fmt(f)?;
                    ']'.fmt(f)?;
                    has_file = true;
                }
                if let Some(line) = line {
                    if *line > -1 {
                        if has_file {
                            ':'.fmt(f)?;
                        }
                        line.fmt(f)?;
                    }
                }
                if let Some(column) = column {
                    if *column > -1 {
                        ':'.fmt(f)?;
                        column.fmt(f)?;
                    }
                }
                if let Some(message) = message {
                    ' '.fmt(f)?;
                    message.fmt(f)?;
                }
                if let Some(stack) = stack {
                    '\n'.fmt(f)?;
                    stack.fmt(f)?;
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
            CaughtError::Exception(e) => RuntimeError::Exception {
                line: e.line(),
                column: e.column(),
                message: e.message(),
                stack: e.stack(),
                file: e.file(),
            },
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
