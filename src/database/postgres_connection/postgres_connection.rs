use ::std::fmt;
use ::std::error::Error;
use ::entities::*;
use super::super::QueryResult;
use super::FromSqlRow;

pub struct PostgresConnection {
    pool: ::r2d2::Pool<::r2d2_postgres::PostgresConnectionManager>,
    null: String
}

pub struct ConnectionStringConfig {
    pub host: String,
    pub port: u64,
    pub user: String,
    pub password: String,
    pub database: String,
    pub identifier: Option<String>
}

// This isn't really how Display is supposed to be implemented, but it gave
// me an excuse to try it
impl fmt::Display for ConnectionStringConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut formatted = format!("postgres://{user}:{password}@{host}:{port}/{database}",
            user = self.user,
            password = self.password,
            host = self.host,
            port = self.port,
            database = self.database
        );
        if let Some(ref identifier) = self.identifier {
            formatted.push_str(&format!("?application_name={}", identifier));
        }
        write!(f, "{}", formatted)
    }
}

impl PostgresConnection {

    pub fn new(connection_settings: ConnectionStringConfig) -> PostgresConnection {
        use ::r2d2_postgres::{TlsMode, PostgresConnectionManager};

        let config = ::r2d2::Config::default();
        let connection_string = format!("{}", connection_settings);
        let manager = PostgresConnectionManager::new(connection_string,
            TlsMode::None).unwrap();
        let _pool = ::r2d2::Pool::new(config, manager).unwrap();
        PostgresConnection {
            pool: _pool,
            null: String::from("NULL")
        }
    }
}

impl super::super::Connection for PostgresConnection {
    fn get_user(&self, id: Option<String>, username: Option<String>) -> QueryResult<User> {
        if id.is_none() && username.is_none() {
            return Err("Must specify at least one argument".to_owned());
        };
        let connection = self.pool.get().unwrap();
        let _id = match id {
            Some(i) => i,
            None => self.null.clone()
        };
        let _username = match username {
            Some(u) => u,
            None => self.null.clone()
        };
        let query = connection.query("SELECT * FROM get_user($1, $2)", &[&_id, &_username]);
        return query.map_err(|e| e.description().to_owned())
        .and_then(|rows| {
            match rows.is_empty() {
                true => Ok(None),
                false => Ok(User::from_sql_row(rows.get(0)))
            }
        });
    }

    fn new_github_user(&self, user: &::entities::GithubUserInfo) -> QueryResult<User> {
        let connection =  self.pool.get().unwrap();
        let query = connection.query("SELECT * FROM create_github_user($1, $2, $3)", &[&user.login, &user.id.to_string(), &user.name]);
        return query.map_err(|e| e.description().to_owned())
        .and_then(|rows| {
            match rows.is_empty() {
                true => Ok(None),
                false => Ok(User::from_sql_row(rows.get(0)))
            }
        });

    }
}