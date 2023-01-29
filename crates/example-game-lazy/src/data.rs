///! Data which should be in a shared library.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UserId(pub(super) i32);

impl std::ops::Deref for UserId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: UserId,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AppId(pub(super) i32);

impl std::ops::Deref for AppId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct App {
    pub id: AppId,
    pub name: String,
}

#[derive(Clone, Deserialize, Copy, Debug)]
pub enum Role {
    /// Connected as an admin, still, the user should be admin for the apps to be able to modify admin data.
    Admin,
    /// Connected as a user of a specific app.
    User(AppId),
}

#[derive(Clone, Deserialize, Debug)]
pub struct BiscuitInfo {
    pub user_id: UserId,
    pub role: Role,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateEmailPasswordData {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginEmailPasswordData {
    pub email: String,
    pub password_plain: String,
}
