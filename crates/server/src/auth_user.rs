use std::fmt::Display;

use actix_web::{dev::ServiceRequest, web, Error, HttpMessage};
use actix_web_httpauth::extractors::{
    bearer::{BearerAuth, Config},
    AuthenticationError,
};
use biscuit_auth::{
    builder::{fact, Term},
    Biscuit, KeyPair,
};
use serde::Serialize;

use crate::models::user::UserId;
use crate::{models::app::AppId, time::MockableDateTime};

#[derive(PartialEq, Serialize, Eq, Clone, Copy, Debug)]
#[non_exhaustive]
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

#[derive(Clone, Serialize, Debug)]
pub struct BiscuitInfo {
    pub user_id: UserId,
    pub role: Role,
}

impl Display for BiscuitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BiscuitInfo{{user_id: {}, role:{}}}",
            self.user_id.0, self.role
        )
    }
}

#[tracing::instrument(name = "validate biscuit as user or admin", skip_all)]
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let root = req.app_data::<web::Data<KeyPair>>().unwrap();
    let time = req.app_data::<web::Data<MockableDateTime>>().unwrap();
    if let Some(biscuit_info) = Biscuit::from_base64(credentials.token(), |_| root.public())
        .ok()
        .and_then(|biscuit| authorize(&biscuit, time))
    {
        req.extensions_mut().insert(biscuit_info);
        Ok(req)
    } else {
        Err((AuthenticationError::from(Config::default()).into(), req))
    }
}

#[tracing::instrument(name = "validate biscuit as admin", skip_all)]
pub async fn validator_admin(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let root = req.app_data::<web::Data<KeyPair>>().unwrap();
    let time = req.app_data::<web::Data<MockableDateTime>>().unwrap();
    if let Some(biscuit_info) = Biscuit::from_base64(credentials.token(), |_| root.public())
        .ok()
        .and_then(|biscuit| authorize(&biscuit, time.get_ref()))
    {
        if biscuit_info.role == Role::Admin {
            req.extensions_mut().insert(biscuit_info);
            return Ok(req);
        }
        Err((AuthenticationError::from(Config::default()).into(), req))
    } else {
        Err((AuthenticationError::from(Config::default()).into(), req))
    }
}

pub fn authorize(token: &Biscuit, time: &MockableDateTime) -> Option<BiscuitInfo> {
    let mut authorizer = token.authorizer().ok()?;
    let time_fact = fact(
        "time",
        &[Term::Date(time.now_utc().unix_timestamp() as u64)],
    );
    authorizer.add_fact(time_fact).map_err(|_| ()).ok()?;
    authorizer.allow().map_err(|_| ()).ok()?;
    authorizer.authorize().map_err(|_| ()).ok()?;

    BiscuitInfo::try_from(&mut authorizer)
        .map_err(|_| "failed ")
        .ok()
}
