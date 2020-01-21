use crate::database::Database;
use diesel::{
    r2d2::{ConnectionManager, CustomizeConnection, Error as R2D2Error, Pool},
    Connection,
};
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    time::Duration,
};

pub struct Builder<C: 'static>
where
    C: Connection,
{
    pub(crate) phantom: PhantomData<C>,
    pub(crate) pool_max_size: Option<u32>,
    pub(crate) pool_min_idle: Option<Option<u32>>,
    pub(crate) pool_max_lifetime: Option<Option<Duration>>,
    pub(crate) on_acquire: Option<Box<dyn Fn(&mut C) -> Result<(), R2D2Error> + Send + Sync>>,
    pub(crate) on_release: Option<Box<dyn Fn(C) + Send + Sync>>,
}

impl<C> Builder<C>
where
    C: Connection,
{
    #[inline]
    pub fn pool_max_size(&mut self, max_size: u32) -> &mut Self {
        self.pool_max_size = Some(max_size);
        self
    }

    #[inline]
    pub fn pool_min_idle(&mut self, min_idle: Option<u32>) -> &mut Self {
        self.pool_min_idle = Some(min_idle);
        self
    }

    #[inline]
    pub fn pool_max_lifetime(&mut self, max_lifetime: Option<Duration>) -> &mut Self {
        self.pool_max_lifetime = Some(max_lifetime);
        self
    }

    #[inline]
    pub fn on_acquire(
        &mut self,
        on_acquire: impl Fn(&mut C) -> Result<(), R2D2Error> + 'static + Send + Sync,
    ) -> &mut Self {
        self.on_acquire = Some(Box::new(on_acquire));
        self
    }

    #[inline]
    pub fn on_release(&mut self, on_release: impl Fn(C) + 'static + Send + Sync) -> &mut Self {
        self.on_release = Some(Box::new(on_release));
        self
    }

    pub fn open(&mut self, url: impl Into<String>) -> Database<C> {
        let manager = ConnectionManager::<C>::new(url);
        let mut p = Pool::builder();

        if let Some(max_size) = self.pool_max_size {
            p = p.max_size(max_size);
        }

        if let Some(min_idle) = self.pool_min_idle {
            p = p.min_idle(min_idle);
        }

        if let Some(max_lifetime) = self.pool_max_lifetime {
            p = p.max_lifetime(max_lifetime);
        }

        if self.on_acquire.is_some() || self.on_release.is_some() {
            p = p.connection_customizer(Box::new(FnConnectionCustomizer {
                on_acquire: self.on_acquire.take(),
                on_release: self.on_release.take(),
            }));
        }

        let pool = p.build_unchecked(manager);

        Database::new(pool)
    }
}

struct FnConnectionCustomizer<C: 'static> {
    on_acquire: Option<Box<dyn Fn(&mut C) -> Result<(), R2D2Error> + Send + Sync>>,
    on_release: Option<Box<dyn Fn(C) + Send + Sync>>,
}

impl<C> Debug for FnConnectionCustomizer<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FnConnectionCustomizer")
    }
}

impl<C> CustomizeConnection<C, R2D2Error> for FnConnectionCustomizer<C> {
    #[inline]
    fn on_acquire(&self, conn: &mut C) -> Result<(), R2D2Error> {
        if let Some(on_acquire) = &self.on_acquire {
            (on_acquire)(conn)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn on_release(&self, conn: C) {
        if let Some(on_release) = &self.on_release {
            (on_release)(conn)
        }
    }
}
