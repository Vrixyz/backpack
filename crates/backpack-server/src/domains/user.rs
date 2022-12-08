use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UserId(i32);

impl std::ops::Deref for UserId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserInput {
    pub name: String,
}

async fn create_user(connection: web::Data<PgPool>, user: web::Json<UserInput>) -> impl Responder {
    if let Ok(user_id) = user.0.create(&connection).await {
        HttpResponse::Ok().json(user_id)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub(crate) fn user() -> impl HttpServiceFactory {
    let cors = Cors::default()
        .allow_any_header()
        .allow_any_origin()
        .allow_any_method()
        .send_wildcard()
        .max_age(3600);
    web::scope("api/v1/users")
        .wrap(cors)
        .route("", web::post().to(create_user))
}

impl UserInput {
    pub async fn create(&self, connection: &PgPool) -> Result<UserId, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO users (name) VALUES ($1)
            RETURNING id
            "#,
            self.name,
        )
        .fetch_one(connection)
        .await?;

        Ok(UserId(rec.id))
    }
}

impl User {
    pub async fn exist(connection: &PgPool, user_id: UserId) -> bool {
        sqlx::query!("SELECT id FROM users WHERE id = $1", *user_id)
            .fetch_one(connection)
            .await
            .is_ok()
    }
    pub async fn get(id: UserId, connection: &PgPool) -> Option<User> {
        sqlx::query!(
            r#"
            SELECT id, name FROM users WHERE id = $1
            "#,
            id.0,
        )
        .fetch_one(connection)
        .await
        .map(|r| User {
            id: UserId(r.id),
            name: r.name.clone(),
        })
        .ok()
    }
}
