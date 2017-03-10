use ::database::Connection;
use std::sync::{Arc,RwLock};
use ::iron::{Handler, IronResult, Response, Request, status};
use plugin::Pluggable;
use ::params::{Params, Value};

pub struct RegisterLocalLogin<C: Connection>{
    connection: Arc<C>
}

impl<C: Connection> RegisterLocalLogin<C> {
    pub fn new(_connection: Arc<C>) -> RegisterLocalLogin<C> {
        RegisterLocalLogin {
            connection: _connection
         }
    }
}

impl<C: Connection + 'static> Handler for RegisterLocalLogin<C> {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {

        let params = request.get_ref::<Params>().unwrap();

        if let (Some(&Value::String(ref username)), Some(&Value::String(ref password))) = (params.find(&["username"]), params.find(&["password"])) {
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

            let user = self.connection.create_local_user(&_username, password, fullname);
            println!("{:?}", user);
        }

        Ok(Response::with((status::BadRequest, "Missing parameter \"username\" or \"password\"")))
    }
}