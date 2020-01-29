#[macro_use]
extern crate diesel;

mod builder;
mod error;
mod database;
mod dsl;
#[cfg(feature = "pg")]
mod paginate;

pub use builder::Builder;
pub use error::AsyncError;
pub use database::{Database, spawn_on_thread};
pub use dsl::AsyncRunQueryDsl;
#[cfg(feature = "pg")]
pub use paginate::*;

pub use threadpool::ThreadPool;