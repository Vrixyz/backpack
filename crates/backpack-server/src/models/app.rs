use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::user::UserId;

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AppId(pub i32);

impl std::ops::Deref for AppId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct App {
    pub id: AppId,
    pub name: String,
}
#[derive(Serialize, Deserialize)]
pub struct AppAdmin {
    pub user_id: UserId,
    pub app_id: AppId,
}

#[derive(Serialize)]
pub struct AppWithName {
    pub name: String,
    pub app_id: AppId,
}

impl AppAdmin {
    pub async fn create_app_admin_relation(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let _rec = sqlx::query!(
            r#"
    INSERT INTO apps_admins ( user_id, app_id )
    VALUES ( $1, $2)
            "#,
            *self.user_id,
            *self.app_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl AppId {
    pub async fn exist(&self, connection: &PgPool) -> bool {
        sqlx::query!("SELECT id FROM apps WHERE id = $1", **self)
            .fetch_one(connection)
            .await
            .is_ok()
    }
    pub async fn get(&self, connection: &PgPool) -> Option<App> {
        sqlx::query!(
            r#"
            SELECT id, name FROM apps WHERE id = $1
            "#,
            **self,
        )
        .fetch_one(connection)
        .await
        .map(|r| App {
            id: AppId(r.id),
            name: r.name.clone(),
        })
        .ok()
    }
    pub async fn create(connection: &PgPool, name: &str) -> Result<AppId, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO apps (name) VALUES ($1)
            RETURNING id
            "#,
            name,
        )
        .fetch_one(connection)
        .await?;

        Ok(AppId(rec.id))
    }

    pub async fn get_all_for_user(
        user: UserId,
        pool: &PgPool,
    ) -> Result<Vec<AppWithName>, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
        SELECT app_id, name
        FROM apps_admins
        JOIN apps
        ON apps.id = app_id
        WHERE user_id = $1
            "#,
            *user,
        )
        .fetch_all(pool)
        .await?;

        Ok(rec
            .into_iter()
            .map(|r| AppWithName {
                name: r.name,
                app_id: AppId(r.app_id),
            })
            .collect())
    }
    pub async fn delete(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                DELETE FROM apps
                WHERE id = $1;
            "#,
            self.0,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_all_for_item(
        pool: &PgPool,
        item_id: super::item::ItemId,
    ) -> Result<Vec<AppWithName>, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
        SELECT apps.id, apps.name
        FROM apps
        JOIN items
        ON items.app_id = apps.id
        WHERE items.id = $1
            "#,
            *item_id,
        )
        .fetch_all(pool)
        .await?;

        Ok(rec
            .into_iter()
            .map(|r| AppWithName {
                name: r.name,
                app_id: AppId(r.id),
            })
            .collect())
    }
}
