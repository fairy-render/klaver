use rquickjs::CaughtError;

#[derive(Debug)]
pub enum Error {
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{:?}", *msg),
            Error::Quick(e) => write!(f, "{e}"),
            Error::Custom(e) => write!(f, "{e}"),
            Error::Exception {
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

impl std::error::Error for Error {}

impl<'js> From<CaughtError<'js>> for Error {
    fn from(value: CaughtError<'js>) -> Self {
        match value {
            CaughtError::Error(err) => err.into(),
            CaughtError::Exception(e) => Error::Exception {
                line: e.line(),
                column: e.column(),
                message: e.message(),
                stack: e.stack(),
                file: e.file(),
            },
            CaughtError::Value(e) => Error::Message(e.as_string().and_then(|m| m.to_string().ok())),
        }
    }
}

impl From<rquickjs::Error> for Error {
    fn from(value: rquickjs::Error) -> Self {
        Error::Quick(value)
    }
}
