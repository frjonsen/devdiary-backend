#[derive(Serialize, Deserialize, Debug, PartialEq)]
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