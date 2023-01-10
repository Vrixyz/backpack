use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::{app::AppId, user::UserId};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ItemId(pub i32);

impl std::ops::Deref for ItemId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ItemId {
    pub async fn delete(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                DELETE FROM items
                WHERE id = $1;
            "#,
            self.0,
        )
        .fetch_one(pool)
        .await?;
        Ok(())
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
                item: ItemWithName {
                    id: ItemId(item.id),
                    name: item.name.clone(),
                },
                amount: item.amount,
            })
            .collect())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ItemFull {
    pub id: ItemId,
    pub name: String,
    pub app_id: AppId,
}
#[derive(Serialize, Deserialize)]
pub struct ItemWithName {
    pub id: ItemId,
    pub name: String,
}
#[derive(Serialize, Deserialize)]
pub struct ItemAmount {
    pub item: ItemWithName,
    pub amount: i32,
}

pub async fn create(name: &str, app_id: AppId, connection: &PgPool) -> Result<ItemId, sqlx::Error> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO items (name, app_id) VALUES ($1, $2)
        RETURNING id
        "#,
        name,
        *app_id,
    )
    .fetch_one(connection)
    .await?;

    Ok(ItemId(rec.id))
}

impl ItemFull {
    pub async fn get(id: ItemId, connection: &PgPool) -> Option<ItemFull> {
        sqlx::query!(
            r#"
            SELECT id, name, app_id FROM items WHERE id = $1
            "#,
            id.0,
        )
        .fetch_one(connection)
        .await
        .map(|r| ItemFull {
            id: ItemId(r.id),
            name: r.name,
            app_id: AppId(r.app_id),
        })
        .ok()
    }
}
