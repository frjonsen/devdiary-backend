use ::database::Connection;
use ::hyper::Client;
use ::hyper::header::{Headers, Accept, UserAgent};
use ::hyper::net::HttpsConnector;
use ::hyper_native_tls::NativeTlsClient;
use ::iron::{Handler, IronResult, Response, Request, status};
use ::plugin::Pluggable;
use ::urlencoded::UrlEncodedQuery;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
struct AccessCode {
    access_token: String,
    token_type: String,
    scope: String
}

#[derive(Serialize, Deserialize, Debug)]
struct GithubUserInfo {
    login: String,
    id: i64,
    name: String
}

pub struct OAuthCallback<C: Connection> {
    http_client: Client,
    reply: String,
    connection: Arc<C>
}

impl<C: Connection> OAuthCallback<C> {
    pub fn new(_connection: Arc<C>) -> OAuthCallback<C> {

        let reply = format!("client_id={}&client_secret={}&code=",
            ::CONFIG.read().unwrap().get_str("github.client_id").unwrap(),
            ::CONFIG.read().unwrap().get_str("github.client_secret").unwrap()
        );

        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);

        OAuthCallback {
            http_client: Client::with_connector(connector),
            reply: reply,
            connection: _connection
         }
    }

    fn get_user_info(&self, token: AccessCode) -> Result<GithubUserInfo, String> {
        use ::std::io::Read;
        use ::std::error::Error;

        let url = format!("https://api.github.com/user?access_token={}", token.access_token);
        let mut headers = Headers::new();
        headers.set(Accept::json());
        headers.set(UserAgent("DevDiary".to_owned()));
        self.http_client.get(&url)
        .headers(headers)
        .send()
        .map_err(|e| e.description().to_owned())
        .and_then(|res| ::serde_json::from_iter::<::std::io::Bytes<_>, GithubUserInfo>(res.bytes())
        .map_err(|e| e.description().to_owned()))
    }

    fn save_new_user(&self, user: GithubUserInfo) -> Result<String, String> {
        println!("{:?}", user);
        Ok("Save successful".to_owned())
    }

    fn handle_access_reply(&self, response: ::hyper::client::Response) -> Result<AccessCode, String> {
        use ::std::io::Read;
        use ::std::error::Error;

        let deserialized: ::serde_json::Result<AccessCode> = ::serde_json::from_iter(response.bytes());
        deserialized.map_err(|e| e.description().to_owned())
    }

    fn access_code_reply(&self, code: &str) -> Result<AccessCode, String> {
        use ::std::error::Error;
        let body = format!("{}{}", self.reply, code);

        let mut headers = Headers::new();
        headers.set(Accept::json());

        self.http_client.post("https://github.com/login/oauth/access_token")
        .body(&body)
        .headers(headers)
        .send()
        .map_err(|e| e.description().to_owned())
        .and_then(|res| self.handle_access_reply(res))
    }
}

impl<C: Connection + 'static> Handler for OAuthCallback<C> {

    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        use ::std::error::Error;
        let result = request.get_ref::<UrlEncodedQuery>()
        .map_err(|e| e.description().to_owned())
        .and_then(|hashmap| hashmap.get("code").unwrap().get(0).ok_or("Parameter \"code\" missing".to_owned()))
        .and_then(|code| self.access_code_reply(code))
        .and_then(|access_code| self.get_user_info(access_code))
        .and_then(|user| self.save_new_user(user));

        match result {
            Ok(res) => Ok(Response::with((status::Ok, res))),
            Err(err) => Ok(Response::with((status::BadRequest, err)))
        }
    }
}