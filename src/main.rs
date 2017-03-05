#[cfg(test)] #[macro_use] extern crate yup_hyper_mock as hyper_mock;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;
extern crate config;
extern crate hyper;
extern crate hyper_native_tls;
extern crate iron;
extern crate iron_sessionstorage;
extern crate logger;
extern crate plugin;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate router;
extern crate serde_json;
extern crate slog_stdlog;
extern crate slog_stream;
extern crate time;
extern crate urlencoded;
extern crate uuid;

mod database;
mod default_config;
mod restserver;
mod entities;
use database::postgres_connection::{PostgresConnection,ConnectionStringConfig};
use default_config::DefaultConfig;
use restserver::{RouterBuilder, Server};
use slog::DrainExt;
use std::io;
use std::sync::RwLock;

lazy_static!{
    static ref CONFIG: RwLock<DefaultConfig> = {
        let mut conf = config::Config::new();
        conf.merge(config::File::new("conf", config::FileFormat::Toml)).unwrap();
        if let Some(config) = conf.get_str("configs.secrets") {
            conf.merge(config::File::new(&config, config::FileFormat::Toml)).unwrap();
        }
        RwLock::new(DefaultConfig::new(conf))
    };
}

struct MyFormat;

impl slog_stream::Format for MyFormat {
    fn format(&self, io: &mut io::Write, rinfo: &slog::Record, _logger_values: &slog::OwnedKeyValueList) -> io::Result<()> {
        let msg = format!("{} - {}\n", rinfo.level(), rinfo.msg());
        let _ = try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}

fn build_connection_settings() -> ConnectionStringConfig {
    let config = CONFIG.read().unwrap();
    ConnectionStringConfig {
        host: config.get_str_or_default("database.host", "localhost"),
        port: (config.get_int_or_default("database.port", 5432i64) as u64),
        user: config.get_str("database.username").unwrap(),
        password: config.get_str_or_default("database.password", ""),
        database: config.get_str("database.database").unwrap(),
        identifier: config.get_str("database.identifier")
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
    let settings = build_connection_settings();
    let router = RouterBuilder::new(PostgresConnection::new(settings))
    .oauth()
    .finalize();
    let s = Server::new(router);
    s.start();
}
