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

use crate::biscuit::parse_biscuit_info;
use crate::time::MockableDateTime;
use shared::{BiscuitInfo, Role};

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
    authorizer.add_fact(dbg!(time_fact)).map_err(|_| ()).ok()?;
    authorizer.allow().map_err(|_| ()).ok()?;
    dbg!("allowed");
    authorizer
        .authorize()
        .map_err(|err| {
            dbg!(err);
            ()
        })
        .ok()?;
    dbg!("authorized");
    parse_biscuit_info(&mut authorizer)
        .map_err(|err| dbg!(err))
        .ok()
}

#[tracing::instrument(name = "decode biscuit but doesn't check inner data", skip_all)]
pub async fn validator_no_check(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let root = req.app_data::<web::Data<KeyPair>>().unwrap();
    let time = req.app_data::<web::Data<MockableDateTime>>().unwrap();
    if let Some(biscuit_info) = Biscuit::from_base64(credentials.token(), |_| root.public())
        .ok()
        .and_then(|biscuit| decode_without_authorization(&biscuit, time))
    {
        req.extensions_mut().insert(biscuit_info);
        Ok(req)
    } else {
        Err((AuthenticationError::from(Config::default()).into(), req))
    }
}

pub fn decode_without_authorization(
    token: &Biscuit,
    time: &MockableDateTime,
) -> Option<BiscuitInfo> {
    let mut authorizer = token.authorizer().ok()?;
    let time_fact = fact(
        "time",
        &[Term::Date(time.now_utc().unix_timestamp() as u64)],
    );
    authorizer.add_fact(time_fact).map_err(|_| ()).ok()?;
    authorizer.allow().map_err(|_| ()).ok()?;

    parse_biscuit_info(&mut authorizer)
        .map_err(|_| "failed ")
        .ok()
}
