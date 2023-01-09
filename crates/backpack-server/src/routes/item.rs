use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::item::create;

#[derive(Deserialize, Serialize)]
pub struct ItemInput {
    pub name: String,
}

async fn create_item(connection: web::Data<PgPool>, item: web::Json<ItemInput>) -> impl Responder {
    if let Ok(item_id) = create(&item.0.name, &connection).await {
        HttpResponse::Ok().json(item_id)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub(crate) fn item() -> impl HttpServiceFactory {
    let cors = Cors::default()
        .allow_any_header()
        .allow_any_origin()
        .allow_any_method()
        .send_wildcard()
        .max_age(3600);
    web::scope("api/v1/items")
        .wrap(cors)
        .route("", web::post().to(create_item))
}
