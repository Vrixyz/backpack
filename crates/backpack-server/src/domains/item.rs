use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::user::UserId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ItemId(i32);

impl std::ops::Deref for ItemId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UserId {
    pub async fn get_items(&self, pool: &PgPool) -> Result<Vec<ItemAmount>, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
        SELECT  item_id as id, amount, items.name as name
        FROM users_items
        JOIN items
        ON items.id = item_id
        WHERE user_id = $1
            "#,
            **self,
        )
        .fetch_all(pool)
        .await?;

        Ok(rec
            .iter()
            .map(|item| ItemAmount {
                item: Item {
                    id: ItemId(item.id),
                    name: item.name.clone(),
                },
                amount: item.amount,
            })
            .collect())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
}
#[derive(Serialize, Deserialize)]
pub struct ItemAmount {
    pub item: Item,
    pub amount: i32,
}

pub async fn create(name: &str, connection: &PgPool) -> Result<ItemId, sqlx::Error> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO items (name) VALUES ($1)
        RETURNING id
        "#,
        self.name,
    )
    .fetch_one(connection)
    .await?;

    Ok(ItemId(rec.id))
}

impl Item {
    pub async fn get(id: ItemId, connection: &PgPool) -> Option<Item> {
        sqlx::query!(
            r#"
            SELECT id, name FROM items WHERE id = $1
            "#,
            id.0,
        )
        .fetch_one(connection)
        .await
        .map(|r| Item {
            id: ItemId(r.id),
            name: r.name,
        })
        .ok()
    }
}
