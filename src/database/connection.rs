pub trait Connection: Send +  Sync{

    /// Returns a user with the given id and username. At least on argument must
    /// be specified.
    ///
    /// # Arguments
    /// * `id` - id of the user, may be None
    /// * `username - username of the user, may be None
    fn get_user(&self, id: Option<String>, username: Option<String>) -> super::QueryResult<::entities::User>;

    fn new_github_user(&self, user: &::entities::GithubUserInfo) -> super::QueryResult<::entities::User>;
}