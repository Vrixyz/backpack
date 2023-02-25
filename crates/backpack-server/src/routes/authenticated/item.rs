use std::fmt::Display;

use actix_web::web::ReqData;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;

use crate::auth_user::BiscuitInfo;
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

impl Display for UserItemModify {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.amount)
    }
}

#[tracing::instrument(
    name = "Get item",
    skip_all,
    fields(item_id=%&*item_id)
)]
async fn get_item(connection: web::Data<PgPool>, item_id: web::Path<i32>) -> impl Responder {
    if let Some(item_full) = ItemFull::get(ItemId(*item_id), &connection).await {
        HttpResponse::Ok().json(item_full)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
    name = "Get user items",
    skip_all,
    fields(user_id=%&*user_id)
)]
/// For a given user, returns all its existing items.
async fn get_user_items(connection: web::Data<PgPool>, user_id: web::Path<i32>) -> impl Responder {
    let user_id = UserId(*user_id);
    if let Ok(res) = user_id.get_items(&connection).await {
        HttpResponse::Ok().json(res)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
    name = "Get user item",
    skip_all,
    fields(user_id=%user_id_item_id.0, item_id=%user_id_item_id.1)
)]
async fn get_user_item(
    connection: web::Data<PgPool>,
    user_id_item_id: web::Path<(i32, i32)>,
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
/// It does not check yet:
/// - :construction: If the item is allowed to be modified by the app the user is authenticated on
/// - :construction: If item is allowed to be modified by user
///
#[tracing::instrument(
    name = "Modify item",
    skip_all,
    fields(biscuit=%&*biscuit, user_item_modify=%&*user_item_modify)
)]
async fn modify_item(
    connection: web::Data<PgPool>,
    user_id_item_id: web::Path<(i32, i32)>,
    biscuit: ReqData<BiscuitInfo>,
    user_item_modify: web::Json<UserItemModify>,
) -> impl Responder {
    let user = biscuit.user_id;
    let item_id = ItemId(user_id_item_id.1);
    match biscuit.role {
        crate::auth_user::Role::Admin => {
            let authorized_apps = AppId::get_all_for_item(&connection, item_id).await.unwrap();
            if !AppId::get_all_for_user(user, &connection)
                .await
                .unwrap()
                .iter()
                .any(|app| {
                    authorized_apps
                        .iter()
                        .any(|authorized_app| authorized_app.app_id == app.app_id)
                })
            {
                return HttpResponse::Unauthorized()
                    .body("You're not admin of any app owner of this item.");
            }
        }
        crate::auth_user::Role::User(app_id) => {
            if biscuit.user_id.0 != user_id_item_id.0 {
                return HttpResponse::Unauthorized()
                    .body("You are not allowed to modify other users' items (yet).");
            }
            let authorized_apps = AppId::get_all_for_item(&connection, item_id).await.unwrap();
            if !authorized_apps
                .iter()
                .any(|authorized_app| authorized_app.app_id == app_id)
            {
                return HttpResponse::Unauthorized()
                    .body("The app does not have rights to modify this item.");
            }
        }
    }
    if let Ok(new_amount) = item_id
        .modify_amount(user, user_item_modify.amount, &connection)
        .await
    {
        HttpResponse::Ok().json(new_amount)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
    name = "Get app items",
    skip_all,
    fields(app_id=%&*app_id)
)]
async fn get_app_items(connection: web::Data<PgPool>, app_id: web::Path<i32>) -> impl Responder {
    let app_id = AppId(*app_id);
    if let Ok(res) = ItemWithName::get_for_app(&connection, app_id).await {
        HttpResponse::Ok().json(res)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
