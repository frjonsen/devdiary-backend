extern crate iron;
extern crate time;
extern crate logger;
extern crate config;
extern crate hyper_native_tls;
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate slog;
extern crate slog_stream;
extern crate slog_stdlog;
#[macro_use]
extern crate log;
use std::sync::RwLock;
use default_config::DefaultConfig;
use hyper_native_tls::NativeTlsServer;

mod server;
mod default_config;

use std::io;
use slog::DrainExt;

lazy_static!{
    static ref CONFIG: RwLock<DefaultConfig> = {
        let mut conf = config::Config::new();
        conf.merge(config::File::new("conf", config::FileFormat::Toml)).unwrap();
        RwLock::new(DefaultConfig::new(conf))
    };
}

struct MyFormat;

impl slog_stream::Format for MyFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              _logger_values: &slog::OwnedKeyValueList)
              -> io::Result<()> {
        let msg = format!("{} - {}\n", rinfo.level(), rinfo.msg());
        let _ = try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}

/// This function will expect the 'file' and 'format' arguments to be present in the config
fn setup_logging() {
    let file = CONFIG.read().unwrap().get_str_or_default("logging.file", "log.log");

    use std::fs::OpenOptions;
    let f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .unwrap();
    let drain = slog_stream::stream(f, MyFormat).fuse();
    let logger = slog::Logger::root(drain, o!());
    slog_stdlog::set_logger(logger).unwrap();
}

fn main() {
    setup_logging();
    let s = server::Server::new();
    s.start();
}
