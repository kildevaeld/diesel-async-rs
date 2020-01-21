use std::fmt::Debug;
use futures_channel::oneshot::Canceled;
use failure::Fail;

#[derive(Debug, Fail)]
pub enum AsyncError<E>
where
    E: 'static + Debug + Send + Sync,
{
    // Timed out trying to checkout a connection
    #[fail(display = "Timeout error: {}", _0)]
    Timeout(r2d2::Error),
    // An error occurred when interacting with the database
    #[fail(display = "Execute error: {:?}", _0)]
    Execute(E),
    #[fail(display = "Cancel error")]
    Canceled
}



impl<E> From<Canceled> for AsyncError<E> 
where
    E: 'static + Debug + Send + Sync,
{
    fn from(_error: Canceled) -> AsyncError<E> {
        AsyncError::Canceled
    }
}
