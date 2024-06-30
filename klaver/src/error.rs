use rquickjs::CaughtError;

#[derive(Debug)]
pub enum Error {
    Quick(rquickjs::Error),
    Unknown(Option<String>),
    Exception {
        line: Option<i32>,
        column: Option<i32>,
        message: Option<String>,
        stack: Option<String>,
    },
}

impl<'js> From<CaughtError<'js>> for Error {
    fn from(value: CaughtError<'js>) -> Self {
        match value {
            CaughtError::Error(err) => err.into(),
            CaughtError::Exception(e) => Error::Exception {
                line: e.line(),
                column: e.column(),
                message: e.message(),
                stack: e.stack(),
            },
            CaughtError::Value(e) => Error::Unknown(e.as_string().and_then(|m| m.to_string().ok())),
        }
    }
}

impl From<rquickjs::Error> for Error {
    fn from(value: rquickjs::Error) -> Self {
        Error::Quick(value)
    }
}
