use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserEmail {
    pub id: i64,
    pub user_id: i64,
    pub email: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserPassword {
    pub id: i64,
    pub user_id: i64,
    pub password: String,
    pub created_at: i64,
}