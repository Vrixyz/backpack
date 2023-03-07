use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;

use crate::models::user::UserId;

pub(crate) fn config() -> impl HttpServiceFactory {
    web::scope("/user").route("/{user_id}", web::get().to(get_user))
}
#[derive(Serialize)]
struct Identity<'a> {
    user_id: &'a UserId,
    name: String,
}

#[tracing::instrument(
    name = "Get user",
    skip_all,
    fields(user_id=%&*user_id)
)]
async fn get_user(connection: web::Data<PgPool>, user_id: web::Path<i32>) -> impl Responder {
    let user_id = UserId(*user_id);
    HttpResponse::Ok().json(Identity {
        user_id: &user_id,
        name: user_id.get(&connection).await.unwrap().name,
    })
}
