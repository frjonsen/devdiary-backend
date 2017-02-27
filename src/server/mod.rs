use iron::prelude::*;
use iron::{BeforeMiddleware,AfterMiddleware, typemap};
use logger::{Logger, Format};
use default_config::DefaultConfig;
pub struct Server;

struct ResponseTime;

impl typemap::Key for ResponseTime { type Value = u64; }

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(super::time::precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = super::time::precise_time_ns() - req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(res)
    }
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((super::iron::status::Ok, "Hello World")))
}

impl Server {
    pub fn new() -> Server {
        Server { }
    }

    fn make_chain(&self) -> Chain {
        let default_format = "Uri: {uri}, Method: {method}, Status: {status}, Duration: {response-time}, Time: {request-time}";
        let format = super::CONFIG.read().unwrap().get_str_or_default("logging.format", &default_format);
        let log_format = Format::new(&format);

        let mut chain = Chain::new(hello_world);
        let (logger_before, logger_after) = Logger::new(log_format);
        chain.link_before(logger_before);
        chain.link_before(ResponseTime);
        chain.link_after(ResponseTime);
        chain.link_after(logger_after);
        chain
    }

    pub fn start(&self) {
        let server = Iron::new(self.make_chain());
        if let Some(identity) = super::CONFIG.read().unwrap().get_str("tls.p12") {
            use hyper_native_tls::NativeTlsServer;
            use std::path::Path;

            let p = Path::new(&identity);
            // openssl req -x509 -newkey rsa:4096 -nodes -keyout localhost.key -out localhost.crt -days 3650
            // openssl pkcs12 -export -out identity.p12 -inkey localhost.key -in localhost.crt --password mypass
            let ssl = NativeTlsServer::new(p, "").unwrap();
            server.https("localhost:3000", ssl);
        }
        else {
            server.http("localhost:3000").unwrap();
        }

    }
}
