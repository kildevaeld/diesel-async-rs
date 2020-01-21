use futures_channel::oneshot::Canceled;
use std::error::Error;
use std::fmt::{self, Debug, Display};

#[derive(Debug)]
pub enum AsyncError<E>
where
    E: 'static + Send + Sync + Error,
{
    // Timed out trying to checkout a connection
    Timeout(r2d2::Error),
    // An error occurred when interacting with the database
    Execute(E),
    Canceled,
}

impl<E> Display for AsyncError<E>
where
    E: 'static + Send + Sync + Error,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Timeout(err) => <r2d2::Error as Display>::fmt(err, f),
            Self::Execute(err) => <E as Display>::fmt(err, f),
            Self::Canceled => write!(f, "Canceled error"),
        }
    }
}

impl<E> Error for AsyncError<E> where E: 'static + Send + Sync + Error {}

impl<E> From<Canceled> for AsyncError<E>
where
    E: 'static + Error + Send + Sync,
{
    fn from(_error: Canceled) -> AsyncError<E> {
        AsyncError::Canceled
    }
}
