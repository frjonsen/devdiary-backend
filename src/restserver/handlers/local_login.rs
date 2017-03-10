use ::database::Connection;
use std::sync::{Arc,RwLock};
use ::iron::{Handler, IronResult, Response, Request, status};

pub struct LocalLogin<C: Connection>{
    connection: Arc<C>
}

impl<C: Connection> LocalLogin<C> {
    pub fn new(_connection: Arc<C>) -> LocalLogin<C> {
        LocalLogin {
            connection: _connection
         }
    }
}

impl<C: Connection + 'static> Handler for LocalLogin<C> {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::NoContent)))
    }
}