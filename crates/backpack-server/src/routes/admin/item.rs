use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    auth_user::{validator_admin, BiscuitInfo},
    models::{
        app::AppId,
        item::{create, ItemFull, ItemId},
    },
};

pub fn config() -> impl HttpServiceFactory {
    web::scope("/item")
        .route("create/app/{app_id}", web::post().to(create_item))
        .route("{item_id}", web::get().to(get_item))
        .route("{item_id}", web::delete().to(delete_item))
}

#[derive(Deserialize, Serialize)]
pub struct ItemInput {
    pub name: String,
}

async fn create_item(
    connection: web::Data<PgPool>,
    item: web::Json<ItemInput>,
    req: HttpRequest,
    app_id: web::Path<i32>,
) -> impl Responder {
    let Some(user) = req.extensions().get::<BiscuitInfo>().map(|b| {b.user_id}) else {
        return HttpResponse::Unauthorized().body("Bad biscuit");
    };
    let Ok(owned_apps) = AppId::get_all_for_user(user, &connection).await else {
        return HttpResponse::Unauthorized().body("no apps for user");
    };
    if owned_apps
        .iter()
        .find(|app| app.app_id.0 == *app_id)
        .is_none()
    {
        return HttpResponse::Unauthorized().body("app not authorized for user");
    }
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
    // TODO: check no user have this item, Please refer to openapi spec for more details.
    return HttpResponse::NotImplemented().finish();
    /*
    if let Ok(_) = ItemId(*item_id).delete(&connection).await {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::InternalServerError().finish()
    }
    */
}
