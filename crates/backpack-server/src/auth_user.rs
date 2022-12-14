use actix_web::{dev::ServiceRequest, web, Error, HttpMessage};
use actix_web_httpauth::extractors::{
    bearer::{BearerAuth, Config},
    AuthenticationError,
};
use biscuit_auth::{Biscuit, KeyPair};

use crate::domains::app::AppId;
use crate::domains::user::UserId;

#[derive(Clone, Copy, Debug)]
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
    dbg!(credentials.token());
    if let Some(biscuit_info) = dbg!(Biscuit::from_base64(credentials.token(), |_| root.public()))
        .ok()
        .and_then(|biscuit| {
            dbg!(&biscuit);
            authorize(&biscuit)
        })
    {
        dbg!(biscuit_info.user_id);
        req.extensions_mut().insert(biscuit_info);
        Ok(req)
    } else {
        Err((AuthenticationError::from(Config::default()).into(), req))
    }
}

pub fn authorize(token: &Biscuit) -> Option<BiscuitInfo> {
    let mut authorizer = token.authorizer().ok()?;

    authorizer.set_time();
    authorizer.allow().map_err(|_| ()).ok()?;
    authorizer.authorize().map_err(|_| ()).ok()?;

    dbg!(BiscuitInfo::try_from(&mut authorizer))
        .map_err(|_| "failed ")
        .ok()
}
