use iron::{Request, Response, IronResult, Handler};
use ::database::Connection;
use router::{Router};
use std::sync::Arc;
use ::iron_sessionstorage::SessionRequestExt;
use iron_sessionstorage::Value;
use ::std::time::SystemTime;
use restserver::handlers::*;

pub struct RouterBuilder<C: Connection> {
    internal_router: Router,
    connection: Arc<C>
}

fn hello_world(request: &mut Request) -> IronResult<Response> {
    /*use ::entities::Session;
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
    }*/
    Ok(Response::with((::iron::status::Ok, "Hello, you were redirected")))
}

impl<C: Connection + 'static> RouterBuilder<C> {
    pub fn new(_connection: Arc<C>) -> RouterBuilder<C> {
        let mut router = Router::new();
        router.get("/", hello_world, "index");
        RouterBuilder {
            internal_router: router,
            connection: _connection
        }
    }

    pub fn oauth(mut self) -> RouterBuilder<C> {
        self.internal_router.get("/oauthcallback", OAuthCallback::new(self.connection.clone()), "oauth_callback");
        self
    }

    pub fn local_users(mut self) -> RouterBuilder<C> {
        self.internal_router.post("/register", RegisterLocalLogin::new(self.connection.clone()), "register_local_user");
        self.internal_router.post("/login", LocalLogin::new(self.connection.clone()), "local_login");
        self
    }

    pub fn finalize(self) -> Router {
        self.internal_router
    }
}