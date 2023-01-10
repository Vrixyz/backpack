use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    auth_user::validator_admin,
    models::{
        app::AppId,
        item::{create, ItemFull, ItemId, ItemWithName},
    },
};

pub(crate) fn item() -> impl HttpServiceFactory {
    let cors = Cors::default()
        .allow_any_header()
        .allow_any_origin()
        .allow_any_method()
        .send_wildcard()
        .max_age(3600);
    web::scope("api/v1")
        .wrap(HttpAuthentication::bearer(validator_admin))
        .wrap(cors)
        .route("/app/{app_id}/item", web::post().to(create_item))
        .route("/item/{item_id}", web::get().to(get_item))
        .route("/item/{item_id}", web::delete().to(delete_item))
}

#[derive(Deserialize, Serialize)]
pub struct ItemInput {
    pub name: String,
}

async fn create_item(
    connection: web::Data<PgPool>,
    app_id: web::Path<i32>,
    item: web::Json<ItemInput>,
) -> impl Responder {
    if let Ok(item_id) = create(&item.0.name, AppId(*app_id), &connection).await {
        HttpResponse::Ok().json(item_id)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

async fn get_item(connection: web::Data<PgPool>, item_id: web::Path<i32>) -> impl Responder {
    if let Some(item_full) = ItemFull::get(ItemId(*item_id), &connection).await {
        HttpResponse::Ok().json(item_full)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
async fn delete_item(connection: web::Data<PgPool>, item_id: web::Path<i32>) -> impl Responder {
    if let Ok(_) = ItemId(*item_id).delete(&connection).await {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
