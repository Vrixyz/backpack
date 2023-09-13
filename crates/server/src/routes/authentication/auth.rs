use std::fmt::Display;

use actix_web::{dev::HttpServiceFactory, web, HttpResponse};
use actix_web_httpauth::{
    extractors::{basic::Config, bearer::BearerAuth, AuthenticationError},
    middleware::HttpAuthentication,
};
use biscuit_auth::{Biscuit, KeyPair};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use shared::{AuthenticationResponse, RefreshToken, Role};
use sqlx::PgPool;
use time::Duration;

use crate::{
    auth_user::{decode_without_authorization, validator, validator_no_check},
    biscuit::{AUTHENTICATION_TOKEN_TTL, REFRESH_TOKEN_TTL},
    models::{self, app::AppId},
    time::MockableDateTime,
};
use shared::RefreshTokenString;

use crate::models::user::UserId;

pub fn config(
    kp: web::Data<KeyPair>,
    time: web::Data<MockableDateTime>,
) -> impl HttpServiceFactory {
    web::scope("/auth")
        .app_data(time)
        .app_data(kp)
        .wrap(HttpAuthentication::bearer(validator_no_check))
        .route("refresh", web::post().to(refresh_authentication_token))
}

#[derive(Debug, Deserialize, Clone)]
pub struct RefreshAuthenticationTokenData {
    pub refresh_token: RefreshTokenString,
}

impl Display for RefreshAuthenticationTokenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.refresh_token)
    }
}

#[tracing::instrument(
    name = "refresh_authentication_token",
    skip_all,
    fields(req_data=%&*req_data)
)]
pub(super) async fn refresh_authentication_token(
    bearer_token: BearerAuth,
    req_data: web::Json<RefreshAuthenticationTokenData>,
    connection: web::Data<PgPool>,
    root: web::Data<KeyPair>,
    time: web::Data<MockableDateTime>,
) -> HttpResponse {
    // TODO: decode bearer token
    let Some(biscuit_info) = Biscuit::from_base64(bearer_token.token(), |_| root.public())
        .ok()
        .and_then(|biscuit| decode_without_authorization(&biscuit, &time))
    else {
        return HttpResponse::InternalServerError().finish();
    };
    let Ok(refresh_token) = models::refresh_token::RefreshToken::get(&connection, &req_data.refresh_token, UserId::from(biscuit_info.user_id)).await else {
        return HttpResponse::BadRequest().finish();
    };
    if refresh_token.revoked {
        // token reuse? Is that a malicious usage?
        return HttpResponse::BadRequest().finish();
    }
    if dbg!(refresh_token.expiration_date) < dbg!(time.now_utc()) {
        return HttpResponse::Forbidden().finish();
    }
    // TODO: #19 refresh token should be returned and revoked in a same DB request (or transaction).
    let Ok(_) = models::refresh_token::RefreshToken::revoke(&connection, refresh_token.id).await else {
        return HttpResponse::Forbidden().finish();
    };

    create_new_authentication_token(
        connection,
        root,
        time,
        UserId::from(biscuit_info.user_id),
        biscuit_info.role.to_option().map(AppId::from),
    )
    .await
}

pub(super) async fn create_new_authentication_token(
    connection: web::Data<PgPool>,
    root: web::Data<KeyPair>,
    time: web::Data<MockableDateTime>,
    user_id: UserId,
    as_app_user: Option<AppId>,
) -> HttpResponse {
    let time_now = time.now_utc();
    let auth_expiration_date = time_now + Duration::seconds(AUTHENTICATION_TOKEN_TTL);
    let biscuit = match as_app_user {
        Some(app_id) => user_id.create_biscuit(&root, Role::User(app_id.0), auth_expiration_date),
        None => user_id.create_biscuit(&root, Role::Admin, auth_expiration_date),
    };
    let refresh_token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(255)
        .map(char::from)
        .collect();
    let Ok(refresh_token) = models::refresh_token::RefreshToken::create(
    connection.as_ref(),
        RefreshTokenString(refresh_token),
        user_id,
        time_now + Duration::seconds(REFRESH_TOKEN_TTL),
        time_now)
     .await else {
        return HttpResponse::InternalServerError().finish();
    };
    let authentication_token = AuthenticationResponse {
        auth_token: biscuit.to_base64().unwrap(),
        refresh_token: RefreshToken {
            refresh_token: refresh_token.refresh_token,
            expiration_date_unix_timestamp: refresh_token.expiration_date.unix_timestamp(),
        },
        expiration_date_unix_timestamp: auth_expiration_date.unix_timestamp(),
    };
    HttpResponse::Ok().json(authentication_token)
}
