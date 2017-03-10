use ::database::Connection;
use std::sync::{Arc,RwLock};
use ::iron::{Handler, IronResult, Response, Request, status};
use plugin::Pluggable;
use ::params::{Params, Value};
use ::entities::{User,Session};
use iron_sessionstorage::SessionRequestExt;
use std::error::Error;

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
}

impl<C: Connection + 'static> Handler for RegisterLocalLogin<C> {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let mut user: ::database::QueryResult<User> = Ok(None);
        {
            let params = request.get_ref::<Params>().unwrap();

            if let (Some(&Value::String(ref username)), Some(&Value::String(ref password))) = (params.find(&["username"]).clone(), params.find(&["password"])) {
                let _username = username.trim().to_owned();
                if _username.trim().is_empty() || password.trim().is_empty() {
                    return Ok(Response::with((status::BadRequest, "Missing parameter \"username\" or \"password\"")));
                }
                let fullname = match params.find(&["fullname"]) {
                    Some(&Value::String(ref name)) => match name.trim().is_empty() {
                        true => None,
                        false => Some(name.trim().to_owned())
                    },
                    _ => None
                };

                user = self.connection.create_local_user(&_username, &password, fullname);
            }
        }
        println!("{:?}", user);
        if let Ok(Some(u)) = user {
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