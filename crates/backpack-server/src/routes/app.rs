use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth_user::validator;

use crate::models::app::AppAdmin;
use crate::models::{app::AppId, user::UserId};

pub(crate) fn app_admin() -> impl HttpServiceFactory {
    web::scope("api/v1")
        .wrap(HttpAuthentication::bearer(validator))
        .route("app", web::post().to(create_app))
}

async fn create_app(
    user_id: web::ReqData<UserId>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    let app_id = AppId::create(&connection, "Placeholder app").await.unwrap();
    AppAdmin {
        user_id: *user_id,
        app_id,
    }
    .create_app_admin_relation(&connection)
    .await
    .unwrap();
    HttpResponse::Created()
}
