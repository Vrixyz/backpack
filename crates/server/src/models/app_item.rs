use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth_user::validator;

use super::{item::ItemId, user::UserId};

#[derive(Serialize, Deserialize)]
pub struct AppItem {
    pub app_id: AppId,
    pub item_id: ItemId,
    pub amount: i32,
}

impl UserItem {
    async fn create_app_items_relation(&self, pool: &PgPool) -> Result<i32, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
    INSERT INTO app_items ( app_id, item_id, amount )
    VALUES ( $1, $2, $3 )
            "#,
            *self.app_id,
            *self.item_id
        )
        .fetch_one(pool)
        .await?;

        Ok(rec.amount)
    }
}
