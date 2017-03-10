use ::database::Connection;
use ::entities::User;
use ::iron_sessionstorage::SessionStorage;
use ::iron_sessionstorage::backends::SignedCookieBackend;
use ::iron_sessionstorage::cookie::Cookie;
use iron::Handler;
use iron::prelude::*;
use iron::{BeforeMiddleware,AfterMiddleware, typemap};
use logger::{Logger, Format};
use router::Router;
use std::sync::Arc;

/*
struct ResponseTime;
impl typemap::Key for ResponseTime { type Value = u64; }

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(::time::precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = ::time::precise_time_ns() - req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(res)
    }
}
*/

struct SessionMiddleware<C: Connection> {
    connection: Arc<C>
}

impl typemap::Key for User {
    type Value = User;
}

impl<C: Connection + 'static> BeforeMiddleware for SessionMiddleware<C> {

    fn before(&self, req: &mut Request) -> IronResult<()> {
        use ::entities::Session;
        use iron_sessionstorage::{SessionRequestExt,Value};
        if let Some(session) = try!(req.session().get::<Session>()) {
            let sess: String = session.into_raw();
            let connection = self.connection.clone();
            let valid_seconds = ::CONFIG.read().unwrap().get_int("sessions.duration");
            if let Ok(Some(user)) = connection.verify_session(&sess, valid_seconds) {
                req.extensions.insert::<User>(user);
            }
        }
        Ok(())
    }
}

pub struct Server<C: Connection, H: Handler> {
    internal_server: Iron<H>,
    connection: Arc<C>
}

impl<C: Connection + 'static> Server<C, Chain> {
    pub fn new(_connection: Arc<C>, router: Router) -> Server<C, Chain> {
        use iron::AroundMiddleware;

        let mut chain = Server::<C, Chain>::make_chain(router, _connection.clone());

        let server = Iron::new(chain);

        Server {
            internal_server: server,
            connection: _connection
         }
    }

    fn read_server_address() -> String {
        let config = ::CONFIG.read().unwrap();
        let port = config.get_str_or_default("server.port", "3000");
        let domain = config.get_str_or_default("server.domain", "localhost");
        return [domain, port].join(":");
    }

    fn read_cookies_secret() -> Vec<u8> {
        let secret = ::CONFIG.read().unwrap().get_str("cookies.secret").unwrap();
        let as_bytes =  secret.as_bytes();
        as_bytes.to_vec()
    }

    fn set_session_cookie_attrs(mut cookie: ::iron_sessionstorage::cookie::Cookie) -> Cookie {
        if let Some(valid_seconds) = ::CONFIG.read().unwrap().get_int("sessions.duration") {
            use ::std::ops::Add;
            let duration = ::time::Duration::seconds(valid_seconds);
            let expires = ::time::now().add(duration);
            cookie.expires = Some(expires);
        }
        cookie
    }

    fn make_chain(router: Router, _connection: Arc<C>) -> Chain {
        let default_format = "Uri: {uri}, Method: {method}, Status: {status}, Duration: {response-time}, Time: {request-time}";
        let format = ::CONFIG.read().unwrap().get_str_or_default("logging.format", &default_format);
        let log_format = Format::new(&format);
        let mut chain = Chain::new(router);
        let sess = SessionMiddleware {
            connection: _connection
        };
        chain.link_before(sess);
        // Need to create a new chain in order to ensure everything is done in the correct order
        let mut c = Chain::new(chain);

        let mut backend = SignedCookieBackend::new(Server::<C, Chain>::read_cookies_secret());
        backend.set_cookie_modifier(Server::<C, Chain>::set_session_cookie_attrs);
        let session_storage = SessionStorage::new(backend);
        c.link_around(session_storage);

        let (logger_before, logger_after) = Logger::new(log_format);
        c.link_before(logger_before);
        //chain.link_before(ResponseTime);
        //chain.link_after(ResponseTime);
        c.link_after(logger_after);
        c
    }

    pub fn start(self) {

        let address: &str = &Server::<C, Chain>::read_server_address();
        if let Some(identity) = ::CONFIG.read().unwrap().get_str("tls.p12") {
            let p = ::std::path::Path::new(&identity);
            // openssl req -x509 -newkey rsa:4096 -nodes -keyout localhost.key -out localhost.crt -days 3650
            // openssl pkcs12 -export -out identity.p12 -inkey localhost.key -in localhost.crt --password mypass
            let ssl = ::hyper_native_tls::NativeTlsServer::new(p, "").unwrap();
            println!{"Listening on https://{}", address}
            self.internal_server.https(address, ssl).unwrap();
        }
        else {
            println!{"Listening on http://{}", address}
            self.internal_server.http(address).unwrap();
        }

    }
}