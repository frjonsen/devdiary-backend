use ::database::Connection;
use ::entities::{User,Session};
use ::iron::{Handler, IronResult, Response, Request, status};
use ::params::{Params, Value};
use iron_sessionstorage::SessionRequestExt;
use plugin::Pluggable;
use std::error::Error;
use std::sync::Arc;

pub struct RegisterLocalLogin<C: Connection>{
    connection: Arc<C>
}

impl<C: Connection> RegisterLocalLogin<C> {
    pub fn new(_connection: Arc<C>) -> RegisterLocalLogin<C> {
        RegisterLocalLogin {
            connection: _connection
         }
    }

    fn create_session(&self, user: &User) -> Result<Session, String> {
        self.connection.create_session(&user)
        .and_then(|res| res.ok_or("Failed to create session for unknown reason".to_owned()))
    }

    fn create_new_user(&self, request: &mut Request) -> Result<User, String> {
            let params = request.get_ref::<Params>().unwrap();

            if let (Some(&Value::String(ref username)), Some(&Value::String(ref password))) = (params.find(&["username"]).clone(), params.find(&["password"])) {
                let _username = username.trim().to_owned();
                if _username.is_empty() || password.trim().is_empty() {
                    return Err("Missing parameter \"username\" or \"password\"".to_owned());
                }
                let fullname = match params.find(&["fullname"]) {
                    Some(&Value::String(ref name)) => match name.trim().is_empty() {
                        true => None,
                        false => Some(name.trim().to_owned())
                    },
                    _ => None
                };

                match self.connection.create_local_user(&_username, &password, fullname) {
                    Ok(Some(user)) => Ok(user),
                    Ok(None) => Err("Failed to create user with no error".to_owned()),
                    Err(err) => Err(err)
                }
            } else {
                Err("Missing parameter \"username\" or \"password\"".to_owned())
            }

    }
}

impl<C: Connection + 'static> Handler for RegisterLocalLogin<C> {

    fn handle(&self, request: &mut Request) -> IronResult<Response> {

        if let Some(_) = request.extensions.get::<User>() {
            return Ok(Response::with((status::Accepted)));
        }

        let user: Result<User, String> = self.create_new_user(request);
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
        };

        Ok(Response::with((status::BadRequest, "Missing parameter \"username\" or \"password\"")))
    }
}