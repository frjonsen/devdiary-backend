use ::entities::{Session, User, GithubUserInfo};
use super::QueryResult;

pub trait Connection: Send + Sync{

    /// Returns a user with the given id and username. At least on argument must
    /// be specified.
    ///
    /// # Arguments
    /// * `id` - id of the user, may be None
    /// * `username - username of the user, may be None
    fn get_user(&self, id: Option<String>, username: Option<String>) -> QueryResult<::entities::User>;

    /// Creates or gets a github user, using the supplied info. If a user with the specified id already exists,
    /// the existing user will be returned instead. This function can thus be used both for registration and login
    ///
    /// # Arguments
    /// * `user`: The information, from github, used to create or get a user
    ///
    fn new_github_user(&self, user: &GithubUserInfo) -> QueryResult<::entities::User>;

    /// Creates a new session for the specified user.
    ///
    /// # Arguments
    /// * `user` - The user to create a new session for
    fn create_session(&self, user: &User) -> QueryResult<::entities::Session>;

    /// Verifies an existing session, fetching the user with the token id
    ///
    /// # Arguments
    /// * `token`- The token string to verify
    fn verify_session(&self, token: &String, duration: Option<i64>) -> QueryResult<::entities::User>;
}