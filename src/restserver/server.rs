use iron::prelude::*;
use iron::{BeforeMiddleware,AfterMiddleware, typemap};
use logger::{Logger, Format};
use router::Router;
use iron::Handler;

struct ResponseTime;
impl typemap::Key for ResponseTime { type Value = u64; }

pub struct Session {
    pub token: String
}

impl ::iron_sessionstorage::Value for Session {
    fn get_key() -> &'static str {
        "session_token"
    }

    fn into_raw(self) -> String {
        self.token
    }

    fn from_raw(value: String) -> Option<Self> {
        if value.is_empty() {
            None
        } else {
            Some(Session { token: value })
        }

    }
}

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

pub struct Server<H: Handler> {
    internal_server: Iron<H>
}

fn read_cookies_secret() -> Vec<u8> {
    let secret = ::CONFIG.read().unwrap().get_str("cookies.secret").unwrap();
    let as_bytes =  secret.as_bytes();
   as_bytes.to_vec()
}

impl Server<Chain> {
    pub fn new(router: super::RestRouter) -> Server<Chain> {
        let server = Iron::new(Server::make_chain(router));
        Server {
            internal_server: server
         }
    }

    fn make_chain(router: super::RestRouter) -> Chain {
        use ::iron_sessionstorage::SessionStorage;
        use ::iron_sessionstorage::backends::SignedCookieBackend;

        let default_format = "Uri: {uri}, Method: {method}, Status: {status}, Duration: {response-time}, Time: {request-time}";
        let format = ::CONFIG.read().unwrap().get_str_or_default("logging.format", &default_format);
        let log_format = Format::new(&format);

        let mut chain = Chain::new(router.internal_router);
        let (logger_before, logger_after) = Logger::new(log_format);
        chain.link_before(logger_before);
        chain.link_before(ResponseTime);
        chain.link_around(SessionStorage::new(SignedCookieBackend::new(read_cookies_secret())));
        chain.link_after(ResponseTime);
        chain.link_after(logger_after);
        chain
    }

    pub fn start(self) {
        let port = ::CONFIG.read().unwrap().get_str_or_default("server.port", "3000");
        let domain = ::CONFIG.read().unwrap().get_str_or_default("server.domain", "localhost");
        let address: &str = &[domain, port].join(":");
        if let Some(identity) = ::CONFIG.read().unwrap().get_str("tls.p12") {
            let p = ::std::path::Path::new(&identity);
            // openssl req -x509 -newkey rsa:4096 -nodes -keyout localhost.key -out localhost.crt -days 3650
            // openssl pkcs12 -export -out identity.p12 -inkey localhost.key -in localhost.crt --password mypass
            let ssl = ::hyper_native_tls::NativeTlsServer::new(p, "").unwrap();
            self.internal_server.https(address, ssl).unwrap();
        }
        else {
            self.internal_server.http(address).unwrap();
        }

    }
}