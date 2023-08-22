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
        let _rec = sqlx::query!(
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
            .into_iter()
            .map(|item| ItemAmount {
                item: ItemWithName {
                    id: ItemId(item.id),
                    name: item.name,
                },
                amount: item.amount,
            })
            .collect())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ItemFull {
    pub item: ItemWithName,
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

impl ItemAmount {
    pub async fn get(
        pool: &PgPool,
        user_id: UserId,
        item_id: ItemId,
    ) -> Result<ItemAmount, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
        SELECT  item_id as id, amount, items.name as name
        FROM users_items
        JOIN items
        ON items.id = item_id
        WHERE user_id = $1
        AND item_id = $2
            "#,
            *user_id,
            *item_id
        )
        .fetch_one(pool)
        .await?;

        Ok(ItemAmount {
            item: ItemWithName {
                id: ItemId(rec.id),
                name: rec.name.clone(),
            },
            amount: rec.amount,
        })
    }
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
            item: ItemWithName {
                id: ItemId(r.id),
                name: r.name,
            },
            app_id: AppId::from(r.app_id),
        })
        .ok()
    }
}

impl ItemWithName {
    pub async fn get_for_app(
        connection: &PgPool,
        app_id: AppId,
    ) -> Result<Vec<ItemWithName>, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
            SELECT id, name FROM items WHERE app_id = $1
            "#,
            *app_id
        )
        .fetch_all(connection)
        .await?;
        Ok(rec
            .into_iter()
            .map(|r| ItemWithName {
                id: ItemId(r.id),
                name: r.name,
            })
            .collect())
    }
}
