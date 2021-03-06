use ::entities::*;
use super::super::QueryResult;

pub struct MockPostgresConnection;

impl super::super::Connection for MockPostgresConnection {
    fn get_user(&self, _id: Option<String>, _username: Option<String>) -> QueryResult<User> {
        let user = match (_id, _username) {
            (Some(i), Some(u)) => User { id: i, username: u, fullname: Some("testfullname".to_owned()) },
            (Some(i), None) => User { id: i, username: "testusername".to_owned(), fullname: Some("testfullname".to_owned()) },
            (None, Some(u)) => User { id: "testid".to_owned(), username: u, fullname: Some("testfullname".to_owned()) },
            (None, None) => return Err("Must specify at least one argument".to_owned())
        };
        Ok(Some(user))
    }

    fn new_github_user(&self, user: &GithubUserInfo) -> QueryResult<User> {
        Ok(Some(User {
            username: user.login.clone(),
            id: "testid".to_owned(),
            fullname: Some(user.name.clone())
        }))
    }

    fn create_session(&self, user: &User) -> QueryResult<Session> {
        Ok(Some(Session {
            token: "sometoken".to_owned()
        }))
    }

    fn verify_session(&self, token: &String, duration: Option<i64>) -> QueryResult<User> {
        Ok(Some(User {
            id: "someid".to_owned(),
            username: "someusername".to_owned(),
            fullname: Some("a full name".to_owned())
        }))
    }

    fn create_local_user(&self, username: &String, password: &String, fullname: Option<String>) -> QueryResult<User> {
        Ok(Some(User {
            id: "someid".to_owned(),
            username: username.clone(),
            fullname: fullname
        }))
    }

    fn verify_local_user(&self, username: &String, password: &String) -> QueryResult<User> {
        match password.as_ref() {
            "thecorrectpassword" => Ok(Some(User {
                id: "someid".to_owned(),
                username: username.clone(),
                fullname: None
            })),
            _ => Ok(None)
        }
    }
}