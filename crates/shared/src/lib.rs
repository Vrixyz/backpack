use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationToken {
    pub refresh_token: RefreshToken,
    pub raw_biscuit: Vec<u8>,
    pub biscuit_info: BiscuitInfo,
}

// FIXME: pub i32 is pretty bad for type safety, because it allows to create wrong users ; should I split UserIdExisting and UserIdUnreliable (to be used for queries)?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
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
    /// unix timestamp (seconds since 1970)
    pub expiration_date_unix_timestamp: i64,
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
    /// unix timestamp (seconds since 1970)
    pub expiration_date_unix_timestamp: i64,
}
impl RefreshToken {
    pub fn will_expire(&self, at_unix_time_seconds_utc: i64) -> bool {
        self.expiration_date_unix_timestamp <= at_unix_time_seconds_utc
    }
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Debug)]
pub enum Role {
    /// Connected as an admin, still, the user should be admin for the apps to be able to modify admin data.
    Admin,
    /// Connected as a user of a specific app.
    User(AppId),
}
impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Role {
    // TODO: #18 Leverage From rust trait for Role -> Option<AppId>
    pub fn to_option(&self) -> Option<AppId> {
        match self {
            Role::User(app_id) => Some(*app_id),
            Role::Admin => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BiscuitInfo {
    pub expiration_date_unix_timestamp: i64,
    pub user_id: UserId,
    pub role: Role,
}

impl Display for BiscuitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BiscuitInfo{{user_id: {}, role:{:?}, expiration: {}}}",
            self.user_id.0, self.role, self.expiration_date_unix_timestamp
        )
    }
}

impl BiscuitInfo {
    pub fn will_expire(&self, at_unix_time_seconds_utc: i64) -> bool {
        self.expiration_date_unix_timestamp <= at_unix_time_seconds_utc
    }
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
