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