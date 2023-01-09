use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::PgPool;

use crate::auth_user::validator;

use crate::models::user::UserId;
use crate::models::user_item::UserItem;

pub(crate) fn user_item() -> impl HttpServiceFactory {
    web::scope("api/v1")
        .wrap(HttpAuthentication::bearer(validator))
        .route("user/item", web::post().to(modify_amount))
        .route("user/{user_id}/item", web::get().to(get_user_items))
}

/// Attempts to modify an item.
/// It does check:
/// - For item's app owner
///   - if
/// - If the item allows for gains
async fn modify_amount(
    connection: web::Data<PgPool>,
    user_item_increment: web::Json<UserItem>,
) -> impl Responder {
    // TODO: check if user has the right to modify this item.

    //
    if let Ok(user_id) = user_item_increment.0.modify_amount(&connection).await {
        HttpResponse::Ok().json(user_id)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

/// For a given user, returns all the items who have at least 1 amount.
async fn get_user_items(connection: web::Data<PgPool>, user_id: web::Path<i32>) -> impl Responder {
    let user_id = UserId(*user_id);
    if let Ok(res) = user_id.get_items(&connection).await {
        HttpResponse::Ok().json(res)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
