use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::{item::ItemId, user::UserId};

#[derive(Serialize, Deserialize)]
pub struct UserItem {
    pub user_id: UserId,
    pub item_id: ItemId,
    pub amount: i32,
}

async fn increment_amount(
    connection: web::Data<PgPool>,
    user_item_increment: web::Json<UserItem>,
) -> impl Responder {
    if let Ok(user_id) = user_item_increment.0.increment_amount(&connection).await {
        HttpResponse::Ok().json(user_id)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub(crate) fn user_item() -> impl HttpServiceFactory {
    let cors = Cors::default()
        .allow_any_header()
        .allow_any_origin()
        .allow_any_method()
        .send_wildcard()
        .max_age(3600);
    web::scope("api/v1/items")
        .wrap(cors)
        .route("", web::post().to(increment_amount))
}

impl UserItem {
    pub async fn increment_amount(&self, pool: &PgPool) -> Result<i32, sqlx::Error> {
        let rec = self.increment_amount_raw(pool).await;
        match rec {
            Ok(amount) => Ok(amount),
            Err(err) => {
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
