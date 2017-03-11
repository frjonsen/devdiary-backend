use ::database::Connection;
use std::sync::Arc;
use ::iron::{Handler, IronResult, Response, Request, status};
use ::params::{Params, Value, Map};
use plugin::Pluggable;
use ::entities::{User,Session};
use iron_sessionstorage::SessionRequestExt;
use std::error::Error;

pub struct LocalLogin<C: Connection>{
    connection: Arc<C>
}

#[derive(Debug,PartialEq)]
enum LoginError {
    MissingParameters(&'static str),
    IncorrectInformation(&'static str),
    UnknownError(String)
}

impl<C: Connection> LocalLogin<C> {
    pub fn new(_connection: Arc<C>) -> LocalLogin<C> {
        LocalLogin {
            connection: _connection
         }
    }

    fn create_session(&self, user: &User) -> Result<Session, String> {
        self.connection.create_session(&user)
        .and_then(|res| res.ok_or("Failed to create session for unknown reason".to_owned()))
    }

    fn verify_user(&self, params: &Map) -> Result<User, LoginError> {

        if let (Some(&Value::String(ref username)), Some(&Value::String(ref password))) = (params.find(&["username"]).clone(), params.find(&["password"])) {
            let _username = username.trim().to_owned();
            if _username.is_empty() || password.trim().is_empty() {
                return Err(LoginError::MissingParameters("Missing parameter \"username\" or \"password\""));
            }
            return match self.connection.verify_local_user(&username, &password) {
                Ok(Some(user)) => Ok(user),
                Ok(None) => Err(LoginError::IncorrectInformation("Incorrect username or password")),
                Err(err) => Err(LoginError::UnknownError(err))
            };
        }
        return Err(LoginError::MissingParameters("Missing parameter \"username\" or \"password\""));
    }
}

impl<C: Connection + 'static> Handler for LocalLogin<C> {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {

        if let Some(_) = request.extensions.get::<User>() {
            return Ok(Response::with((status::Accepted)));
        }

        let user = self.verify_user(request.get_ref::<Params>().unwrap());
        if let Ok(u) = user {
            let res = match self.create_session(&u) {
                Ok(session) => request.session().set(session).map_err(|e| e.description().to_owned()),
                Err(err) => Err(err)
            };

            return match res {
                Ok(_) => Ok(Response::with((status::NoContent))),
                Err(err) => Ok(Response::with((status::BadRequest, err)))
            };
        } else if let Err(err) = user {
            return match err {
                LoginError::IncorrectInformation(s) => Ok(Response::with((status::UnprocessableEntity, s))),
                LoginError::MissingParameters(s) => Ok(Response::with((status::BadRequest, s))),
                LoginError::UnknownError(s) => Ok(Response::with((status::InternalServerError, s)))
            };
        }
        Ok(Response::with((status::InternalServerError)))
    }
}

#[cfg(test)]
mod test {
    use super::LocalLogin;
    use super::Connection;
    use ::database::postgres_connection::MockPostgresConnection;
    use std::sync::Arc;
    use ::params::{Params, Value, Map};
    use ::iron::request::Request;
    use iron_test::request::post;
    use iron::{Handler, status, headers, Headers};
    use iron::mime::Mime;

    impl LocalLogin<MockPostgresConnection> {
        fn new_test() -> LocalLogin<MockPostgresConnection> {
            let postgres = MockPostgresConnection{};
            let c: Arc<MockPostgresConnection> = Arc::new(postgres);
            LocalLogin {
                connection: c
            }
        }
    }

    #[test]
    fn test_valid_correct_verify_user() {
        let login = LocalLogin::new_test();
        let mut params = Map::new();
        params.assign("username", Value::String("ausername".into()));
        params.assign("password", Value::String("thecorrectpassword".into()));
        let user = login.verify_user(&params).unwrap();
        assert_eq!(user.username, "ausername");
    }

    #[test]
    fn test_valid_incorrect_verify_user() {
        let login = LocalLogin::new_test();
        let mut params = Map::new();
        params.assign("username", Value::String("ausername".into()));
        params.assign("password", Value::String("anincorrectpassword".into()));
        let err = login.verify_user(&params).unwrap_err();
        let expected = super::LoginError::IncorrectInformation("Incorrect username or password");
        assert_eq!(err, expected);
    }

    #[test]
    fn test_invalid_missing_password_verify_user() {
        let login = LocalLogin::new_test();
        let mut params = Map::new();
        params.assign("username", Value::String("ausername".into()));
        let err = login.verify_user(&params).unwrap_err();
        let expected = super::LoginError::MissingParameters("Missing parameter \"username\" or \"password\"");
        assert_eq!(err, expected);
    }

    #[test]
    fn test_invalid_missing_username_verify_user() {
        let login = LocalLogin::new_test();
        let mut params = Map::new();
        params.assign("password", Value::String("anincorrectpassword".into()));
        let err = login.verify_user(&params).unwrap_err();
        let expected = super::LoginError::MissingParameters("Missing parameter \"username\" or \"password\"");
        assert_eq!(err, expected);
    }

    #[test]
    fn test_invalid_empty_username_verify_user() {
        let login = LocalLogin::new_test();
        let mut params = Map::new();
        params.assign("username", Value::String(" ".into()));
        params.assign("password", Value::String("anincorrectpassword".into()));
        let err = login.verify_user(&params).unwrap_err();
        let expected = super::LoginError::MissingParameters("Missing parameter \"username\" or \"password\"");
        assert_eq!(err, expected);
    }

    #[test]
    fn test_full_valid_local_login() {
        let login = LocalLogin::new_test();
        let chain = super::super::add_cookie_backend(login);

        let mut headers = Headers::new();
        let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
        headers.set(headers::ContentType(mime));
        let res = post("https://example.com", headers, "username=ausername&password=thecorrectpassword", &chain).unwrap();
        assert_eq!(res.status.unwrap(), status::NoContent);
    }

    #[test]
    fn test_full_incorrect_local_login() {
        let login = LocalLogin::new_test();
        let chain = super::super::add_cookie_backend(login);

        let mut headers = Headers::new();
        let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
        headers.set(headers::ContentType(mime));
        let res = post("https://example.com", headers, "username=ausername&password=thecorrectpasswor", &chain).unwrap();
        assert_eq!(res.status.unwrap(), status::UnprocessableEntity);
    }

    #[test]
    fn test_full_missing_username_local_login() {
        let login = LocalLogin::new_test();
        let chain = super::super::add_cookie_backend(login);

        let mut headers = Headers::new();
        let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
        headers.set(headers::ContentType(mime));
        let res = post("https://example.com", headers, "password=thecorrectpasswor", &chain).unwrap();
        assert_eq!(res.status.unwrap(), status::BadRequest);
    }

    #[test]
    fn test_full_missing_password_local_login() {
        let login = LocalLogin::new_test();
        let chain = super::super::add_cookie_backend(login);

        let mut headers = Headers::new();
        let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
        headers.set(headers::ContentType(mime));
        let res = post("https://example.com", headers, "username=ausername", &chain).unwrap();
        assert_eq!(res.status.unwrap(), status::BadRequest);
    }
}