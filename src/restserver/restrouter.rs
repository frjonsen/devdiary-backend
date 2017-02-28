use iron::{Request, Response, IronResult, Handler};
use super::database::Connection;
use router::{Router};
use std::sync::Arc;

pub struct RestRouter {
    pub internal_router: Router,
    connection: Arc<Connection>
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((::iron::status::Ok, "Hello World")))
}

impl RestRouter {

    pub fn new<C: Connection + 'static>(_connection: C) -> RestRouter {
        let mut router = Router::new();
        router.get("/", hello_world, "index");
        RestRouter {
            internal_router: router,
            connection: Arc::new(_connection)
        }
    }
}