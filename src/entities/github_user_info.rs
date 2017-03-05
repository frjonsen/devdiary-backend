#[derive(Serialize, Deserialize, Debug)]
pub struct GithubUserInfo {
    pub login: String,
    pub id: i64,
    pub name: String
}