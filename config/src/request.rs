use serde::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    User(UserRequest),
}



#[derive(Debug, Serialize, Deserialize)]
pub enum UserRequest {
    Add { name: String },
    Remove { name: String },
    Rename { old_name: String, new_name: String },
}