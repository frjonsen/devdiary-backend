mod local_login;
mod oauth;
mod register_local_login;

pub use self::local_login::LocalLogin;
pub use self::oauth::OAuthCallback;
pub use self::register_local_login::RegisterLocalLogin;
use iron::Request;

trait UrlForTrait {
    fn get_url_for(&self, request: &Request, target: &str) -> ::iron::Url {
        let iron_url = url_for!(request, target);
        let mut generic_url: ::url::Url = iron_url.into();
        generic_url.set_scheme("https");
        ::iron::Url::from_generic_url(generic_url).unwrap()
    }
}