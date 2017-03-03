pub trait Connection: Send +  Sync{
    //fn create_user(&self, username: String, password: String, fullname: Option<String>);
    fn hello(&self);
}