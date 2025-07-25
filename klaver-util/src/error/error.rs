use rquickjs::CaughtError;

use crate::error::exception::CaugthException;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub enum RuntimeError {
    Quick(rquickjs::Error),
    Custom(BoxError),
    Exception(CaugthException),
}

impl RuntimeError {
    pub fn new<T: Into<BoxError>>(error: T) -> RuntimeError {
        RuntimeError::Custom(error.into())
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::Quick(e) => write!(f, "{e}"),
            RuntimeError::Custom(e) => write!(f, "{e}"),
            RuntimeError::Exception(e) => {
                write!(f, "{e}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Quick(e) => Some(e),
            Self::Custom(e) => Some(&**e),
            Self::Exception(e) => Some(e),
        }
    }
}

impl<'js> From<CaughtError<'js>> for RuntimeError {
    fn from(value: CaughtError<'js>) -> Self {
        RuntimeError::Exception(value.into())
    }
}

impl From<rquickjs::Error> for RuntimeError {
    fn from(value: rquickjs::Error) -> Self {
        RuntimeError::Quick(value)
    }
}

impl From<BoxError> for RuntimeError {
    fn from(value: BoxError) -> Self {
        RuntimeError::Custom(value)
    }
}
