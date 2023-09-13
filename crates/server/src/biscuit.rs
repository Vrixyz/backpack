use std::time::UNIX_EPOCH;

use actix_web::cookie::time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};
use biscuit_auth::{
    builder::{date, fact, BiscuitBuilder, Fact, Term},
    Authorizer, Biscuit, KeyPair,
};
use serde::{Deserialize, Serialize};

use super::models::app::AppId;
use super::models::user::UserId;
use crate::time::MockableDateTime;

use shared::{BiscuitInfo, Role};

pub const AUTHENTICATION_TOKEN_TTL: i64 = 30;
pub const REFRESH_TOKEN_TTL: i64 = 30 * 24 * 3600;

/// Contains a biscuit token.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenReply {
    /// Biscuit token.
    pub token: String,
}

pub struct AuthorizedWrapper<T>(pub T);

impl<T> std::ops::Deref for AuthorizedWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait BiscuitBaker<T> {
    fn bake(builder: BiscuitBuilder, ingredient: T) -> BiscuitBuilder;
}

fn parse_role(authorizer: &mut Authorizer) -> Result<Role, String> {
    let admin: Option<Vec<(bool,)>> = authorizer
        .query("data($is_admin) <- is_admin($is_admin)")
        .ok();
    match admin {
        Some(res) if !res.is_empty() && res[0].0 => Ok(Role::Admin),
        _ => {
            let app_id: Vec<(String,)> = authorizer
                .query("data($app_id) <- user_app_id($app_id)")
                .map_err(|_| "query app_id error")?;
            Ok(Role::User(shared::AppId(
                app_id
                    .get(0)
                    .ok_or("get(0) error")?
                    .0
                    .as_str()
                    .parse::<i32>()
                    .map_err(|_| "parse error")?,
            )))
        }
    }
}

fn parse_user_id(authorizer: &mut Authorizer) -> Result<UserId, String> {
    let res: Vec<(String,)> = authorizer
        .query("data($id) <- user($id)")
        .map_err(|_| "query error")?;
    Ok(UserId::from(
        res.get(0)
            .ok_or("get(0) error")?
            .0
            .as_str()
            .parse::<i32>()
            .map_err(|_| "parse error")?,
    ))
}

fn parse_expiration_date(authorizer: &mut Authorizer) -> Result<i64, String> {
    let res: Vec<(i64,)> = authorizer
        .query("data($unix_timestamp) <- expiration_date($unix_timestamp)")
        .map_err(|err| err.to_string())?;
    Ok(res.get(0).ok_or("get(0) error")?.0)
}

pub fn parse_biscuit_info(authorizer: &mut Authorizer) -> Result<BiscuitInfo, String> {
    Ok(BiscuitInfo {
        expiration_date_unix_timestamp: parse_expiration_date(authorizer)?,
        user_id: parse_user_id(authorizer)?.0,
        role: parse_role(authorizer)?,
    })
}

impl<'a> BiscuitBaker<UserId> for BiscuitBuilder<'a> {
    fn bake(mut builder: BiscuitBuilder, ingredient: UserId) -> BiscuitBuilder {
        builder
            .add_authority_fact(Fact::new(
                "user".to_string(),
                vec![Term::Str((*ingredient).to_string())],
            ))
            .unwrap();
        builder
    }
}

impl<'a> BiscuitBaker<Role> for BiscuitBuilder<'a> {
    fn bake(mut builder: BiscuitBuilder, ingredient: Role) -> BiscuitBuilder {
        match dbg!(ingredient) {
            Role::Admin => {
                builder
                    .add_authority_fact(Fact::new("is_admin".to_string(), vec![Term::Bool(true)]))
                    .unwrap();
            }
            Role::User(app_id) => {
                builder
                    .add_authority_fact(Fact::new(
                        "user_app_id".to_string(),
                        vec![Term::Str((app_id).to_string())],
                    ))
                    .unwrap();
            }
        }
        builder
    }
}

impl UserId {
    pub fn create_biscuit(
        &self,
        root: &KeyPair,
        role: Role,
        expiration_date: OffsetDateTime,
    ) -> Biscuit {
        let mut builder = Biscuit::builder(root);

        builder = BiscuitBuilder::bake(builder, *self);
        builder = BiscuitBuilder::bake(builder, role);
        builder
            .add_authority_fact(
                format!(r#"expiration_date({})"#, expiration_date.unix_timestamp()).as_str(),
            )
            .unwrap();
        builder
            .add_authority_check(
                format!(
                    r#"check if time($time), $time < {}"#,
                    expiration_date.format(&Rfc3339).unwrap()
                )
                .as_str(),
            )
            .unwrap();

        builder.build().unwrap()
    }
}

pub fn authorize_user_only(token: &Biscuit, time: &MockableDateTime) -> Option<UserId> {
    let mut authorizer = token.authorizer().ok()?;

    let time_fact = fact(
        "time",
        &[Term::Date(time.now_utc().unix_timestamp() as u64)],
    );
    authorizer.add_fact(time_fact).map_err(|_| ()).ok()?;
    authorizer.allow().map_err(|_| ()).ok()?;
    authorizer.authorize().map_err(|_| ()).ok()?;

    dbg!(parse_user_id(&mut authorizer)
        .map_err(|_| "authorize error")
        .ok())
}
