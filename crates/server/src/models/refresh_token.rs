use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;
use serde::{Deserialize, Serialize};
use shared::{RefreshTokenId, RefreshTokenString};
use sqlx::PgPool;
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::auth_user::validator;

use super::user::UserId;

#[derive(Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: RefreshTokenId,
    pub refresh_token: RefreshTokenString,
    pub user_id: UserId,
    pub expiration_date: OffsetDateTime,
    pub revoked: bool,
    pub created_at: OffsetDateTime,
}

impl RefreshToken {
    pub async fn create(
        pool: &PgPool,
        refresh_token: RefreshTokenString,
        user_id: UserId,
        expiration_date: OffsetDateTime,
        created_at: OffsetDateTime,
    ) -> Result<RefreshToken, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
    INSERT INTO refresh_tokens ( refresh_token, user_id, expiration_date, revoked, created_at )
    VALUES ( $1, $2, $3, false, $4 ) RETURNING id
            "#,
            &*refresh_token,
            *user_id,
            PrimitiveDateTime::new(expiration_date.date(), expiration_date.time()),
            PrimitiveDateTime::new(created_at.date(), created_at.time())
        )
        .fetch_one(pool)
        .await?;

        Ok(RefreshToken {
            id: RefreshTokenId(rec.id),
            refresh_token,
            user_id,
            expiration_date,
            revoked: false,
            created_at,
        })
    }
    pub async fn get(
        pool: &PgPool,
        refresh_token: &RefreshTokenString,
        user_id: UserId,
    ) -> Result<RefreshToken, sqlx::Error> {
        sqlx::query!(
            r#"
            SELECT 
                id,
                refresh_token,
                user_id,
                expiration_date,
                revoked,
                created_at
            FROM refresh_tokens
            WHERE refresh_token = $1
            AND user_id = $2
            "#,
            &refresh_token.0,
            *user_id,
        )
        .fetch_one(pool)
        .await
        .map(|r| RefreshToken {
            id: RefreshTokenId(r.id),
            refresh_token: RefreshTokenString(r.refresh_token),
            user_id: UserId(r.user_id),
            expiration_date: r.expiration_date.assume_utc(),
            revoked: r.revoked,
            created_at: r.created_at.assume_utc(),
        })
    }
    pub async fn revoke(
        pool: &PgPool,
        refresh_token_id: RefreshTokenId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens SET revoked = true
            WHERE id = $1
            "#,
            refresh_token_id.0
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
