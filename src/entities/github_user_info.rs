#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GithubUserInfo {
    pub login: String,
    pub id: i64,
    pub name: String
}