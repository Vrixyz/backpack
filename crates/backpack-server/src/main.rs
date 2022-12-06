use tokio::task::futures;

use sqlx::{postgres::PgPoolOptions, PgPool};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres@localhost/backpack")
        .await?;

    let user_id = create_user(&pool, "Vrixyz").await?;

    dbg!(user_id);
    Ok(())
}

async fn create_user(pool: &PgPool, username: &str) -> Result<i32, sqlx::Error> {
    let rec = sqlx::query!(
        r#"
INSERT INTO users ( name )
VALUES ( $1 )
RETURNING id
        "#,
        username,
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}
async fn create_item_to_user_relation(
    pool: &PgPool,
    user_id: i32,
    item_id: i32,
) -> Result<i32, sqlx::Error> {
    let rec = sqlx::query!(
        r#"
INSERT INTO users_items ( user_id, item_id )
VALUES ( $1, $2 )
RETURNING amount
        "#,
        user_id,
        item_id
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.amount)
}
