use sqlx::PgPool;

use super::user::UserId;

#[derive(Debug, Clone, Copy)]
pub struct EmailPasswordId(pub i32);

/// Checks if a email exists as an email/password record.
pub async fn exist(connection: &PgPool, email: &str) -> bool {
    sqlx::query!(
        "SELECT id FROM users_email_password WHERE email = $1",
        email
    )
    .fetch_one(connection)
    .await
    .is_ok()
}
/// Checks if a email exists as an email/password record and return its id, password hash and user_id.
pub async fn find(
    connection: &PgPool,
    email: &str,
) -> Result<(EmailPasswordId, String, UserId), sqlx::Error> {
    sqlx::query!(
        "SELECT id, password_hash, user_id FROM users_email_password WHERE email = $1",
        email
    )
    .fetch_one(connection)
    .await
    .map(|rec| {
        (
            EmailPasswordId(rec.id),
            rec.password_hash,
            UserId(rec.user_id),
        )
    })
}
/// Meant to be used with another query following, to link it to this authentication method.
/// FIXME: This API could be reworked to be misuse resistant.
pub async fn create(
    connection: &PgPool,
    email: &str,
    password_hash: &str,
    account: UserId,
) -> bool {
    sqlx::query!(
        r#"
            INSERT INTO users_email_password (email, password_hash, is_verified, user_id) VALUES ($1, $2, $3, $4)
            RETURNING id
        "#,
        email,
        password_hash,
        false,
        *account,
    )
    .fetch_one(connection)
    .await
    .map(|_| true)
    .is_ok()
}
