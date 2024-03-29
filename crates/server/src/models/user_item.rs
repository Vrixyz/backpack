use sqlx::PgPool;

use super::{item::ItemId, user::UserId};

impl ItemId {
    pub async fn modify_amount(
        &self,
        user: UserId,
        amount: i32,
        pool: &PgPool,
    ) -> Result<i32, sqlx::Error> {
        let rec = dbg!(self.increment_amount_raw(user, amount, pool).await);
        match rec {
            Ok(amount) => Ok(amount),
            Err(_err) => dbg!(self.create_item_to_user_relation(user, amount, pool).await),
        }
    }

    async fn create_item_to_user_relation(
        &self,
        user: UserId,
        amount: i32,
        pool: &PgPool,
    ) -> Result<i32, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
    INSERT INTO users_items ( user_id, item_id, amount )
    VALUES ( $1, $2, $3 )
    RETURNING amount
            "#,
            *user,
            self.0,
            amount
        )
        .fetch_one(pool)
        .await?;

        Ok(rec.amount)
    }
    /// Can also be used to subtract.
    async fn increment_amount_raw(
        &self,
        user: UserId,
        amount: i32,
        pool: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<i32, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
UPDATE users_items SET amount = amount + $1
  WHERE user_id = $2 AND item_id = $3
  RETURNING amount
        "#,
            amount,
            *user,
            self.0
        )
        .fetch_one(pool)
        .await?;
        Ok(rec.amount)
    }
}
