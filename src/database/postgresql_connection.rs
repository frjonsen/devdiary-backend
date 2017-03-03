pub struct PostgresqlConnection;

impl super::connection::Connection for PostgresqlConnection {
    fn hello(&self) {
        println!("Hello");
    }
}