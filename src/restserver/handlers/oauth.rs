use ::database::Connection;
use ::entities::{GithubUserInfo,User};
use ::hyper::Client;
use ::hyper::header::{Headers, Accept, UserAgent};
use ::hyper::net::HttpsConnector;
use ::hyper_native_tls::NativeTlsClient;
use ::iron::{Handler, IronResult, Response, Request, status};
use ::plugin::Pluggable;
use ::urlencoded::UrlEncodedQuery;
use std::collections::HashMap;
use std::sync::{Arc,RwLock};

lazy_static! {
    static ref GITHUB_ENDPOINTS: RwLock<HashMap<&'static str, &'static str>> = {
        let mut m = HashMap::new();
        m.insert("user_info", "https://api.github.com/user");
        m.insert("access_code", "https://github.com/login/oauth/access_token");
        RwLock::new(m)
    };
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AccessCode {
    access_token: String,
    token_type: String,
    scope: String
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

        let url = format!("{}?access_token={}", GITHUB_ENDPOINTS.read().unwrap().get("user_info").unwrap(), token.access_token);
        println!("{}", url);
        let mut headers = Headers::new();
        headers.set(Accept::json());
        headers.set(UserAgent("DevDiary".to_owned()));
        self.http_client.get(&url)
        .headers(headers)
        .send()
        .map_err(|e| e.description().to_owned())
        .and_then(|response| {
            match response.status {
                ::hyper::status::StatusCode::Ok => Ok(response),
                _ => {
                    match response.status.canonical_reason() {
                        Some(ref reason) => Err(reason.to_string()),
                        _ => Err("Authentication failed".to_owned())
                    }
                }
            }
        })
        .and_then(|res| ::serde_json::from_iter::<::std::io::Bytes<_>, GithubUserInfo>(res.bytes())
        .map_err(|e| e.description().to_owned()))
    }

    // This should not attempt to create a new user if it already exists
    // (rename postgres function to create_or_get_github_user)
    fn save_new_user(&self, user: GithubUserInfo) -> Result<User, String> {
        self.connection.new_github_user(&user)
        .and_then(|o| o.ok_or("Failed to create user for unknown reason".to_owned()))
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

        self.http_client.post(*GITHUB_ENDPOINTS.read().unwrap().get("access_code").unwrap())
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
            Ok(res) => Ok(Response::with((status::Ok, format!("{:?}", res)))),
            Err(err) => Ok(Response::with((status::BadRequest, err)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::OAuthCallback;
    use super::Connection;
    use ::database::postgres_connection::MockPostgresConnection;
    use std::sync::Arc;
    use ::hyper;
    // This is a terribly ugly way to solve race conditions, but since it's just
    // for the test, it's not worth changing the "real" code to make it work. Decent
    // solution would be to just use a better mocking library, if one existed.
    lazy_static! {
        static ref ENDPOINTS_LOCK: ::std::sync::Mutex<i32> = ::std::sync::Mutex::new(5);
    }

    impl <C: Connection> OAuthCallback<C> {
        fn new_test(_connection: Arc<C>) -> OAuthCallback<C> {
            OAuthCallback {
                http_client: create_mock_http(),
                reply: "Unused in tests".to_owned(),
                connection: _connection
            }
        }
    }

    fn create_mock_http() -> ::hyper::Client {
        let mut urls = super::GITHUB_ENDPOINTS.write().unwrap();
        println!("Set user_info valid");
        urls.insert("access_code", "https://validaccesscodeurl.com");
        mock_connector!(MockRedirectPolicy {
            "https://validaccesscodeurl.com" => "HTTP/1.1 200 OK\r\n\
                                            Server: mock3\r\n\
                                            \r\n\
                                            {
                                            \"access_token\": \"sometoken\",
                                            \"token_type\": \"sometype\",
                                            \"scope\": \"user:email\"
                                            }
                                            "
            "https://validuserinfourl.com"  => "HTTP/1.1 200 OK\r\n\
                                            Server: mock3\r\n\
                                            \r\n\
                                            {
                                            \"login\": \"someusername\",
                                            \"id\": 741852963,
                                            \"name\": \"some realname\"
                                            }
                                            "
            "https://invaliduserinfourl.com" => "HTTP/1.1 404 Not Found\r\n\
                                            Server: mock3\r\n\
                                            \r\n\
                                            {
                                            \"message\": \"Not Found\",
                                            \"documentation_url\": \"https://developer.github.com/v3\"
                                            }
                                            "
        });

        let mut client = ::hyper::Client::with_connector(MockRedirectPolicy::default());
        client.set_redirect_policy(::hyper::client::RedirectPolicy::FollowAll);
        client
    }

    fn create_oauth() -> OAuthCallback<MockPostgresConnection> {
        let mock = MockPostgresConnection{};
        let arc: Arc<MockPostgresConnection> = Arc::new(mock);
        super::OAuthCallback::new_test(arc)
    }

    #[test]
    fn test_valid_access_code_reply() {

        let oauth = create_oauth();
        let reply = oauth.access_code_reply("somecode");
        let expected = super::AccessCode {
            access_token: "sometoken".to_owned(),
            token_type: "sometype".to_owned(),
            scope: "user:email".to_owned()
        };
        assert_eq!(expected, reply.unwrap());
    }

    #[test]
    fn test_valid_user_info_reply() {

        let accesscode = super::AccessCode {
            access_token: "Sometoken".to_owned(),
            token_type: "Notused".to_owned(),
            scope: "Notused".to_owned()
        };
        let expected = super::GithubUserInfo {
            login: "someusername".to_owned(),
            id: 741852963,
            name: "some realname".to_owned()
        };
        let oauth = create_oauth();
        {
            let lock = ENDPOINTS_LOCK.lock();
            super::GITHUB_ENDPOINTS.write().unwrap().insert("user_info", "https://validuserinfourl.com");
            let reply = oauth.get_user_info(accesscode);
            assert_eq!(expected, reply.unwrap());
        }
    }

    #[test]
    fn test_invalid_user_info_reply() {
        let oauth = create_oauth();
        let accesscode = super::AccessCode {
            access_token: "Someothertoken".to_owned(),
            token_type: "Notused".to_owned(),
            scope: "Notused".to_owned()
        };
        {
            let lock = ENDPOINTS_LOCK.lock();
            super::GITHUB_ENDPOINTS.write().unwrap().insert("user_info", "https://invaliduserinfourl.com");
            let reply = oauth.get_user_info(accesscode);
            assert_eq!(reply.unwrap_err(), "Not Found".to_owned());
        }
    }
}