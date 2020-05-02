#[cfg(feature = "pg")]
#[macro_use]
extern crate diesel;

mod builder;
mod database;
mod dsl;
mod error;
#[cfg(feature = "pg")]
mod paginate;

pub use builder::Builder;
pub use database::{spawn_on_thread, Database};
pub use dsl::AsyncRunQueryDsl;
pub use error::AsyncError;
#[cfg(feature = "pg")]
pub use paginate::*;

pub use threadpool::ThreadPool;
