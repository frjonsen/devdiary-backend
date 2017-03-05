pub mod postgres_connection;
mod connection;

pub use self::connection::Connection;
pub use self::postgres_connection::*;

pub type QueryResult<T> = Result<Option<T>, String>;