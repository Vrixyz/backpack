use serde::{Deserialize, Serialize};

// FIXME: pub i32 is pretty bad for type safety, because it allows to create wrong users ; should I split UserIdExisting and UserIdUnreliable (to be used for queries)?
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UserId(pub i32);

impl std::ops::Deref for UserId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ItemId(pub i32);

impl std::ops::Deref for ItemId {
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

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthenticationResponse {
    pub auth_token: String,
    pub refresh_token: RefreshToken,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct RefreshTokenId(pub i32);
impl std::ops::Deref for RefreshTokenId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RefreshTokenString(pub String);

impl std::fmt::Display for RefreshTokenString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0[0..8])
    }
}

impl std::ops::Deref for RefreshTokenString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RefreshToken {
    pub refresh_token: RefreshTokenString,
    /// Format is date time such as RFC3339 https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
    ///
    /// example: `2030-07-21T17:32:28Z`
    pub expiration_date: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AppId(pub i32);

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

#[derive(Clone, PartialEq, Eq, Deserialize, Copy, Debug)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemWithName {
    pub id: ItemId,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAmount {
    pub item: ItemWithName,
    pub amount: i32,
}

// region: request parameters

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateEmailPasswordData {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreatedUserEmailPasswordData {
    pub id: UserId,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginEmailPasswordData {
    pub email: String,
    pub password_plain: String,
    pub as_app_user: Option<AppId>,
}

#[derive(Deserialize, Serialize)]
pub struct UserItemModify {
    pub amount: i32,
}
