use ::database::Connection;
use std::sync::Arc;
use ::iron::{Handler, IronResult, Response, Request, status};
use ::params::{Params, Value};
use plugin::Pluggable;
use ::entities::{User,Session};
use iron_sessionstorage::SessionRequestExt;
use std::error::Error;

pub struct LocalLogin<C: Connection>{
    connection: Arc<C>
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

    fn verify_user(&self, request: &mut Request) -> Result<User, String> {
        let params = request.get_ref::<Params>().unwrap();
        if let (Some(&Value::String(ref username)), Some(&Value::String(ref password))) = (params.find(&["username"]).clone(), params.find(&["password"])) {
            let _username = username.trim().to_owned();
            if _username.is_empty() || password.trim().is_empty() {
                return Err("Missing parameter \"username\" or \"password\"".to_owned());
            }
            return match self.connection.verify_local_user(&username, &password) {
                Ok(Some(user)) => Ok(user),
                Ok(None) => Err("Incorrect username or password".to_owned()),
                Err(err) => Err(err)
            };
        }
        return Err("Missing parameter \"username\" or \"password\"".to_owned());
    }
}

impl<C: Connection + 'static> Handler for LocalLogin<C> {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {

        if let Some(_) = request.extensions.get::<User>() {
            return Ok(Response::with((status::Accepted)));
        }
        let user = self.verify_user(request);

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
            return Ok(Response::with((status::BadRequest, err)));
        }
        Ok(Response::with((status::UnprocessableEntity, "Missing parameter \"username\" or \"password\"".to_owned())))
    }
}