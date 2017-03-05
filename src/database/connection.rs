use ::entities::{Session, User};

pub trait Connection: Send +  Sync{

    /// Returns a user with the given id and username. At least on argument must
    /// be specified.
    ///
    /// # Arguments
    /// * `id` - id of the user, may be None
    /// * `username - username of the user, may be None
    fn get_user(&self, id: Option<String>, username: Option<String>) -> super::QueryResult<::entities::User>;

    /// Creates or gets a github user, using the supplied info. If a user with the specified id already exists,
    /// the existing user will be returned instead. This function can thus be used both for registration and login
    ///
    /// # Arguments
    /// * `user`: The information, from github, used to create or get a user
    ///
    fn new_github_user(&self, user: &::entities::GithubUserInfo) -> super::QueryResult<::entities::User>;

    //fn create_session(&self, user: &::entities::User, duration: u64) -> QueryResult<::entities::Session>;
}