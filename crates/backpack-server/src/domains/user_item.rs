use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth_user::validator;

use super::{item::ItemId, user::UserId};

#[derive(Serialize, Deserialize)]
pub struct UserItem {
    pub user_id: UserId,
    pub item_id: ItemId,
    pub amount: i32,
}

pub(crate) fn user_item(kp: web::Data<KeyPair>) -> impl HttpServiceFactory {
    web::scope("api/v1")
        .app_data(kp)
        .wrap(HttpAuthentication::bearer(validator))
        .route("user/item", web::post().to(modify_amount))
        .route("user/{user_id}/item", web::get().to(get_user_items))
}

/// Attempts to modify an item.
/// It does check:
/// - For item's app owner
///   - if
/// - If the item allows for gains
async fn modify_amount(
    connection: web::Data<PgPool>,
    user_item_increment: web::Json<UserItem>,
) -> impl Responder {
    // TODO: check if user has the right to modify this item.

    //
    if let Ok(user_id) = user_item_increment.0.modify_amount(&connection).await {
        HttpResponse::Ok().json(user_id)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

/// For a given user, returns all the items who have at least 1 amount.
async fn get_user_items(connection: web::Data<PgPool>, user_id: web::Path<i32>) -> impl Responder {
    let user_id = UserId(*user_id);
    if let Ok(res) = user_id.get_items(&connection).await {
        HttpResponse::Ok().json(res)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

impl UserItem {
    pub async fn modify_amount(&self, pool: &PgPool) -> Result<i32, sqlx::Error> {
        let rec = self.increment_amount_raw(pool).await;
        match rec {
            Ok(amount) => Ok(amount),
            Err(_err) => {
                self.create_item_to_user_relation(pool).await?;
                self.increment_amount_raw(pool).await
            }
        }
    }

    async fn create_item_to_user_relation(&self, pool: &PgPool) -> Result<i32, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
    INSERT INTO users_items ( user_id, item_id, amount )
    VALUES ( $1, $2, $3 )
    RETURNING amount
            "#,
            *self.user_id,
            *self.item_id,
            self.amount
        )
        .fetch_one(pool)
        .await?;

        Ok(rec.amount)
    }
    /// Can also be used to subtract.
    async fn increment_amount_raw(
        &self,
        pool: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<i32, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
UPDATE users_items SET amount = amount + $1
  WHERE user_id = $2 AND item_id = $3
  RETURNING amount
        "#,
            self.amount,
            *self.user_id,
            *self.item_id
        )
        .fetch_one(pool)
        .await;
        match rec {
            Ok(rec) => Ok(rec.amount),
            Err(err) => Err(err),
        }
    }
}
