use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web::{HttpMessage, HttpRequest};
use serde::Deserialize;
use sqlx::PgPool;

use crate::auth_user::BiscuitInfo;
use crate::models::app::AppAdmin;
use crate::models::app::AppId;

pub(super) fn config() -> impl HttpServiceFactory {
    web::resource("/app")
        .route(web::post().to(create_app))
        .route(web::get().to(get_apps_for_admin))
        .route(web::delete().to(delete_app))
}

#[derive(Debug, Deserialize, Clone)]
struct CreateAppData {
    pub name: String,
}

async fn create_app(
    connection: web::Data<PgPool>,
    req_data: web::Json<CreateAppData>,
    req: HttpRequest,
) -> impl Responder {
    let Some(user) = req.extensions().get::<BiscuitInfo>().map(|b| {b.user_id}) else {
        return HttpResponse::Unauthorized().finish();
    };
    let app_id = AppId::create(&connection, &req_data.name).await.unwrap();
    AppAdmin {
        user_id: user,
        app_id,
    }
    .create_app_admin_relation(&connection)
    .await
    .unwrap();
    HttpResponse::Created().json(app_id.0)
}

async fn get_apps_for_admin(connection: web::Data<PgPool>, req: HttpRequest) -> impl Responder {
    let Some(user) = req.extensions().get::<BiscuitInfo>().map(|b| {b.user_id}) else {
        return HttpResponse::Unauthorized().finish();
    };

    if let Ok(apps) = AppId::get_all_for_user(user, &connection).await {
        HttpResponse::Ok().json(apps)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[derive(Debug, Deserialize, Clone)]
struct DeleteAppData {
    pub id: i32,
}

async fn delete_app(
    connection: web::Data<PgPool>,
    app_id: web::Json<DeleteAppData>,
    req: HttpRequest,
) -> impl Responder {
    let Some(user) = req.extensions().get::<BiscuitInfo>().map(|b| {b.user_id}) else {
        return HttpResponse::Unauthorized().finish();
    };
    let app = AppId(app_id.id);

    let Ok(apps) = AppId::get_all_for_user(user, &connection).await
    else {
        return HttpResponse::InternalServerError().finish();
    };
    if apps.iter().any(|a| a.app_id == app) {
        app.delete(&connection).await.unwrap();
        return HttpResponse::Ok().finish();
    }
    HttpResponse::Unauthorized().finish()
}
