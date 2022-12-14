use actix_web::{
    cookie::time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime},
    dev::HttpServiceFactory,
    web, HttpResponse, Responder, Scope,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::{
    builder::{BiscuitBuilder, Fact, Term},
    Authorizer, Biscuit, KeyPair,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth_user::{validator, BiscuitInfo, Role};

use super::models::app::AppId;
use super::models::user::UserId;

pub const TOKEN_TTL: i64 = 600;

/// Contains a biscuit token.
#[derive(Serialize, Deserialize)]
pub struct TokenReply {
    /// Biscuit token.
    pub token: String,
}

pub trait BiscuitBaker<T> {
    fn bake(builder: BiscuitBuilder, ingredient: T) -> BiscuitBuilder;
}

impl<'a> TryFrom<&'a mut Authorizer<'a>> for Role {
    type Error = String;

    fn try_from(authorizer: &mut Authorizer) -> Result<Self, Self::Error> {
        let admin: Option<Vec<(bool,)>> = authorizer
            .query("data($is_admin) <- is_admin($is_admin)")
            .ok();
        match admin {
            Some(res) if !res.is_empty() && res[0].0 => Ok(Role::Admin),
            _ => {
                let app_id: Vec<(String,)> = authorizer
                    .query("data($app_id) <- user_app_id($app_id)")
                    .map_err(|_| "query app_id error")?;
                Ok(Role::User(AppId(
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
}

impl<'a> TryFrom<&'a mut Authorizer<'a>> for UserId {
    type Error = String;

    fn try_from(value: &mut Authorizer) -> Result<Self, Self::Error> {
        let res: Vec<(String,)> = value
            .query("data($id) <- user($id)")
            .map_err(|_| "query error")?;
        Ok(UserId(
            res.get(0)
                .ok_or("get(0) error")?
                .0
                .as_str()
                .parse::<i32>()
                .map_err(|_| "parse error")?,
        ))
    }
}
impl<'a, 'b: 'a> TryFrom<&'b mut Authorizer<'a>> for BiscuitInfo {
    type Error = String;

    fn try_from(authorizer: &'a mut Authorizer<'b>) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: UserId::try_from(&mut authorizer.clone())?,
            role: Role::try_from(authorizer)?,
        })
    }
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
        match ingredient {
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
    pub fn create_biscuit(&self, root: &KeyPair, role: Role) -> Biscuit {
        let mut builder = Biscuit::builder(root);

        builder = BiscuitBuilder::bake(builder, *self);
        builder = BiscuitBuilder::bake(builder, role);
        builder
            .add_authority_check(
                format!(
                    r#"check if time($time), $time < {}"#,
                    (OffsetDateTime::now_utc() + Duration::seconds(TOKEN_TTL))
                        .format(&Rfc3339)
                        .unwrap()
                )
                .as_str(),
            )
            .unwrap();

        builder.build().unwrap()
    }
}

pub fn authorize_user_only(token: &Biscuit) -> Option<UserId> {
    let mut authorizer = token.authorizer().ok()?;

    authorizer.set_time();
    authorizer.allow().map_err(|_| ()).ok()?;
    authorizer.authorize().map_err(|_| ()).ok()?;

    UserId::try_from(&mut authorizer)
        .map_err(|_| "authorize error")
        .ok()
}
