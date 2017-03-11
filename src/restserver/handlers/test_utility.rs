use ::iron_sessionstorage::SessionStorage;
use ::iron_sessionstorage::backends::SignedCookieBackend;
use ::iron_sessionstorage::cookie::Cookie;
use ::iron::Chain;

pub fn add_cookie_backend<H: ::iron::Handler>(handler: H) -> Chain {
    let mut c = Chain::new(handler);
    let secret: Vec<u8> = "asecret".as_bytes().to_vec();
    let mut backend = SignedCookieBackend::new(secret);
    let session_storage = SessionStorage::new(backend);
    c.link_around(session_storage);
    c
}