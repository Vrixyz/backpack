use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::models::app::AppId;
use crate::models::item::ItemId;

pub(crate) fn config() -> impl HttpServiceFactory {
    web::scope("/app").route("/item/{item_id}", web::get().to(get_app_item))
}

/// For a given user, returns all its existing items.
async fn get_app_item(connection: web::Data<PgPool>, item_id: web::Path<i32>) -> impl Responder {
    let item_id = ItemId(*item_id);
    if let Ok(res) = AppId::get_all_for_item(&connection, item_id).await {
        HttpResponse::Ok().json(res)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
