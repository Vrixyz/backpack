use actix_web::{
    dev::HttpServiceFactory,
    web::{self, ReqData},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use shared::BiscuitInfo;

use crate::models::{app::AppId, item::create, user::UserId};

pub fn config() -> impl HttpServiceFactory {
    web::scope("/item")
        .route("/app/{app_id}", web::post().to(create_item))
        .route("/{item_id}", web::delete().to(delete_item))
}

#[derive(Deserialize, Serialize)]
pub struct ItemInput {
    pub name: String,
}

#[tracing::instrument(
    name = "Create item",
    skip_all,
    fields(biscuit=%&*biscuit, app_id=&*app_id)
)]
async fn create_item(
    connection: web::Data<PgPool>,
    item: web::Json<ItemInput>,
    biscuit: ReqData<BiscuitInfo>,
    app_id: web::Path<i32>,
) -> impl Responder {
    let user = biscuit.user_id;
    let Ok(owned_apps) = AppId::get_all_for_user(UserId::from(user), &connection).await else {
        return HttpResponse::Unauthorized().body("no apps for user");
    };
    if !owned_apps
        .iter()
        .any(|app| app.app_id == AppId::from(*app_id))
    {
        return HttpResponse::Unauthorized().body("app not authorized for user");
    }
    if let Ok(item_id) = create(&item.0.name, AppId::from(*app_id), &connection).await {
        HttpResponse::Ok().json(item_id)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
    name = "delete item",
    skip_all,
    fields(item_id=%&*_item_id)
)]
async fn delete_item(_connection: web::Data<PgPool>, _item_id: web::Path<i32>) -> impl Responder {
    // TODO: #9 check no user have this item, Please refer to openapi spec for more details.
    HttpResponse::NotImplemented().finish()
    /*
    if let Ok(_) = ItemId(*item_id).delete(&connection).await {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::InternalServerError().finish()
    }
    */
}
