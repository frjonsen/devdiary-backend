#[cfg(test)]pub mod mock_postgres_connection;
#[cfg(test)]pub use self::mock_postgres_connection::MockPostgresConnection;

mod postgres_connection;
pub use self::postgres_connection::PostgresConnection;
pub use self::postgres_connection::ConnectionStringConfig;
use ::entities::*;
use ::postgres::rows::Row;

trait FromSqlRow: Sized {
    fn from_sql_row(row: Row) -> Option<Self>;
}

impl FromSqlRow for User {
    fn from_sql_row(row: Row) -> Option<Self> {

        let _id: ::uuid::Uuid = row.get("id");

        let user = User {
            id: _id.simple().to_string(),
            fullname: row.get("fullname"),
            username: row.get("username")
        };
        Some(user)
    }
}

impl FromSqlRow for Session {
    fn from_sql_row(row: Row) -> Option<Self> {

        Some(Session {
            token: row.get(0),
        })
    }
}