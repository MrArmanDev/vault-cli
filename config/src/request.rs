use serde::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    User(UserRequest),

    Unlock(String),

    Lock,

    Default(String),

    Vault(Vault)
}



#[derive(Debug, Serialize, Deserialize)]
pub enum UserRequest {
    Add { name: String },
    Remove { name: String },
    Rename { old_name: String, new_name: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Vault {
    Add(VaultAdd),
    Get(VaultGet)
}


#[derive(Debug, Serialize, Deserialize)]
pub struct VaultAdd {
    pub username: String,
    pub app: String,
    pub hint: String,
    pub master: Option<String>,
    pub pass: String,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct VaultGet {
    pub username: Option<String>,
    pub app: Option<String>,
    pub master: Option<String>, 

}