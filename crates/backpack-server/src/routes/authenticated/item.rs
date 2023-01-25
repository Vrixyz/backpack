use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web::{HttpMessage, HttpRequest};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;
use serde::Deserialize;
use sqlx::PgPool;

use crate::auth_user::{validator, BiscuitInfo};

use crate::models::app::AppId;
use crate::models::item::{ItemAmount, ItemFull, ItemId, ItemWithName};
use crate::models::user::UserId;

pub(crate) fn config() -> impl HttpServiceFactory {
    web::scope("/item")
        .route("/{item_id}", web::get().to(get_item))
        .route("/user/{user_id}", web::get().to(get_user_items))
        .route("/{item_id}/user/{user_id}", web::get().to(get_user_item))
        .route(
            "/{item_id}/user/{user_id}/modify",
            web::post().to(modify_item),
        )
        .route("/app/{item_id}", web::get().to(get_app_items))
}

#[derive(Deserialize)]
pub struct UserItemModify {
    pub amount: i32,
}

async fn get_item(connection: web::Data<PgPool>, item_id: web::Path<i32>) -> impl Responder {
    if let Some(item_full) = ItemFull::get(ItemId(*item_id), &connection).await {
        HttpResponse::Ok().json(item_full)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
/// For a given user, returns all its existing items.
async fn get_user_items(
    connection: web::Data<PgPool>,
    req: HttpRequest,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id = UserId(*user_id);
    if let Ok(res) = user_id.get_items(&connection).await {
        HttpResponse::Ok().json(res)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

async fn get_user_item(
    connection: web::Data<PgPool>,
    user_id_item_id: web::Path<(i32, i32)>,
    req: HttpRequest,
) -> impl Responder {
    let user_id = UserId(user_id_item_id.0);
    let item_id = ItemId(user_id_item_id.1);
    if let Ok(res) = ItemAmount::get(&connection, user_id, item_id).await {
        HttpResponse::Ok().json(res)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

/// For a authenticated user, modify item.
/// Attempts to modify an item.
/// It does check:
/// - For item's app owner
/// - :construction: If the item allows for gains
async fn modify_item(
    connection: web::Data<PgPool>,
    user_id_item_id: web::Path<(i32, i32)>,
    req: HttpRequest,
    user_item_modify: web::Json<UserItemModify>,
) -> impl Responder {
    let Some(user) = req.extensions().get::<BiscuitInfo>().map(|b| {b.user_id}) else {
        return HttpResponse::Unauthorized().finish();
    };
    let item_id = ItemId(user_id_item_id.1);
    if let Ok(user_id) = item_id
        .modify_amount(user, user_item_modify.amount, &connection)
        .await
    {
        HttpResponse::Ok().json(user_id)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

async fn get_app_items(
    connection: web::Data<PgPool>,
    app_id: web::Path<i32>,
    req: HttpRequest,
) -> impl Responder {
    let app_id = AppId(*app_id);
    if let Ok(res) = ItemWithName::get_for_app(&connection, app_id).await {
        HttpResponse::Ok().json(res)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
