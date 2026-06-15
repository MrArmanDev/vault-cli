use serde::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    User(UserRequest),

    Unlock(String),

    Lock,

    Default(String)
}



#[derive(Debug, Serialize, Deserialize)]
pub enum UserRequest {
    Add { name: String },
    Remove { name: String },
    Rename { old_name: String, new_name: String },
}