use iron::{Request, Response, IronResult, Handler};
use ::database::Connection;
use router::{Router};
use std::sync::Arc;
use ::iron_sessionstorage::SessionRequestExt;
use iron_sessionstorage::Value;
use ::std::time::SystemTime;
use restserver::handlers::*;

pub struct RouterBuilder {
    internal_router: Router,
    connection: Arc<Connection>
}

fn hello_world(request: &mut Request) -> IronResult<Response> {
    use super::session::Session;
    let localtime = ::time::now().rfc822().to_string();
    if let Ok(Some(session)) = request.session().get::<Session>() {
        let sess = session.into_raw();
        println!("Was {:?}", sess);
        request.session().set(Session {
            token: localtime
        });
        Ok(Response::with((::iron::status::Ok, sess)))
    } else {
        println!("Setting to {:?}", localtime);
        request.session().set(Session {
            token: localtime
        });
        Ok(Response::with((::iron::status::Ok, "No session cookie found")))
    }
}

impl RouterBuilder {
    pub fn new<C: Connection + 'static>(_connection: C) -> RouterBuilder {
        let mut router = Router::new();
        router.get("/", hello_world, "index");
        RouterBuilder {
            internal_router: router,
            connection: Arc::new(_connection)
        }
    }

    pub fn oauth(mut self) -> RouterBuilder {
        self.internal_router.get("/oauthcallback", OAuthCallback::new(), "oauth_callback");
        self
    }

    pub fn finalize(mut self) -> Router {
        self.internal_router
    }
}