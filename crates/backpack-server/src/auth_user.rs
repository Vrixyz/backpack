use actix_web::{dev::ServiceRequest, web, Error, HttpMessage};
use actix_web_httpauth::extractors::{
    bearer::{BearerAuth, Config},
    AuthenticationError,
};
use biscuit_auth::{Biscuit, KeyPair};

use crate::models::app::AppId;
use crate::models::user::UserId;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Role {
    /// Connected as an admin, still, the user should be admin for the apps to be able to modify admin data.
    Admin,
    /// Connected as a user of a specific app.
    User(AppId),
}

#[derive(Clone, Debug)]
pub struct BiscuitInfo {
    pub user_id: UserId,
    pub role: Role,
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let root = req.app_data::<web::Data<KeyPair>>().unwrap();
    if let Some(biscuit_info) = Biscuit::from_base64(credentials.token(), |_| root.public())
        .ok()
        .and_then(|biscuit| authorize(&biscuit))
    {
        req.extensions_mut().insert(biscuit_info);
        Ok(req)
    } else {
        Err((AuthenticationError::from(Config::default()).into(), req))
    }
}

pub async fn validator_admin(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let root = req.app_data::<web::Data<KeyPair>>().unwrap();
    if let Some(biscuit_info) = Biscuit::from_base64(credentials.token(), |_| root.public())
        .ok()
        .and_then(|biscuit| authorize(&biscuit))
    {
        if (biscuit_info.role == Role::Admin) {
            req.extensions_mut().insert(biscuit_info);
            return Ok(req);
        }
        return Err((AuthenticationError::from(Config::default()).into(), req));
    } else {
        Err((AuthenticationError::from(Config::default()).into(), req))
    }
}

pub fn authorize(token: &Biscuit) -> Option<BiscuitInfo> {
    let mut authorizer = token.authorizer().ok()?;

    authorizer.set_time();
    authorizer.allow().map_err(|_| ()).ok()?;
    authorizer.authorize().map_err(|_| ()).ok()?;

    BiscuitInfo::try_from(&mut authorizer)
        .map_err(|_| "failed ")
        .ok()
}
