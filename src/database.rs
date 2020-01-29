use crate::{builder::Builder, error::AsyncError};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection,
};
use futures_channel::oneshot::{channel, Canceled, Receiver};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{fmt::Debug, marker::PhantomData};
use threadpool::ThreadPool;

pub struct Database<C: 'static>
where
    C: Connection,
{
    pub(crate) tp: ThreadPool,
    pub(crate) pool: Pool<ConnectionManager<C>>,
}

impl<C: 'static> Clone for Database<C>
where
    C: Connection,
{
    fn clone(&self) -> Self {
        Database {
            tp: self.tp.clone(),
            pool: self.pool.clone(),
        }
    }
}

impl<C> Database<C>
where
    C: Connection,
{
    pub fn new(pool: Pool<ConnectionManager<C>>) -> Database<C> {
        Database::new_with_threadpool(pool, ThreadPool::default())
    }

    pub fn new_with_threadpool(
        pool: Pool<ConnectionManager<C>>,
        threadpool: ThreadPool,
    ) -> Database<C> {
        Database {
            pool,
            tp: threadpool,
        }
    }

    #[inline]
    pub fn open(url: impl Into<String>) -> Database<C> {
        Self::builder().open(url)
    }

    #[inline]
    pub fn builder() -> Builder<C> {
        Builder {
            phantom: PhantomData,
            pool_max_size: None,
            pool_min_idle: None,
            pool_max_lifetime: None,
            on_acquire: None,
            on_release: None,
        }
    }

    /// Executes the given function inside a database transaction.
    #[inline]
    pub fn transaction<F, R, E>(&self, f: F) -> impl Future<Output = Result<R, AsyncError<E>>>
    where
        F: 'static + (FnOnce(&C) -> Result<R, E>) + Send,
        R: 'static + Send,
        E: 'static + From<diesel::result::Error> + std::error::Error + Send + Sync, // + From<TaskError>,
    {
        self.get(move |conn| conn.transaction(|| f(&conn)))
    }

    /// Executes the given function with a connection retrieved from the pool.
    ///
    /// This is non-blocking
    pub fn get<F, R, E>(&self, f: F) -> impl Future<Output = Result<R, AsyncError<E>>>
    where
        F: 'static + Send + FnOnce(&C) -> Result<R, E>,
        R: 'static + Send,
        E: 'static + std::error::Error + Send + Sync,
    {
        let pool = self.pool.clone();
        spawn_on_thread(&self.tp, move || {
            match pool.get() {
                Ok(conn) => f(&conn).map_err(|e| AsyncError::Execute(e)),
                Err(e) => Err(AsyncError::Timeout(e)),
            }  
        }, |_| {})
    }
}

pub fn spawn_on_thread<F, C, R, E>(
    thread: &ThreadPool,
    f: F,
    ocb: C,
) -> impl Future<Output = Result<R, E>>
where
    F: (FnOnce() -> Result<R, E>) + Send + 'static,
    C: FnOnce(Result<R, E>) + Send + 'static,
    R: 'static + Send,
    E: 'static + From<Canceled> + Send + Sync,
{
    let (sx, rx) = channel();
    thread.execute(move|| {
        let ret = f();
        if let Err(ret) = sx.send(ret) {
            ocb(ret);
        }
    });

    ChannelReceiverFuture::new(rx)
}

#[pin_project]
pub struct ChannelReceiverFuture<O, E> {
    #[pin]
    rx: Receiver<Result<O, E>>,
}

impl<O, E> ChannelReceiverFuture<O, E> {
    pub fn new(rx: Receiver<Result<O, E>>) -> ChannelReceiverFuture<O, E> {
        ChannelReceiverFuture { rx }
    }
}

impl<O, E: Send + Sync + 'static + From<Canceled>> Future for ChannelReceiverFuture<O, E> {
    type Output = Result<O, E>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.rx.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(s)) => Poll::Ready(s),
            Poll::Ready(Err(err)) => Poll::Ready(Err(E::from(err))),
        }
    }
}
