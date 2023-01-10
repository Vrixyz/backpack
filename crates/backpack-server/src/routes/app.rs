use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web::{HttpMessage, HttpRequest};
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::PgPool;

use crate::auth_user::{validator_admin, BiscuitInfo};

use crate::models::app::AppAdmin;
use crate::models::{app::AppId, user::UserId};

pub(crate) fn app_admin() -> impl HttpServiceFactory {
    web::scope("api/v1")
        .wrap(HttpAuthentication::bearer(validator_admin))
        .route("app", web::post().to(create_app))
        .route("app", web::get().to(get_apps_for_admin))
}

async fn create_app(connection: web::Data<PgPool>, req: HttpRequest) -> impl Responder {
    let Some(user) = req.extensions().get::<BiscuitInfo>().map(|b| {b.user_id}) else {
        return HttpResponse::Unauthorized().finish();
    };
    let app_id = AppId::create(&connection, "Placeholder app").await.unwrap();
    AppAdmin {
        user_id: user,
        app_id,
    }
    .create_app_admin_relation(&connection)
    .await
    .unwrap();
    HttpResponse::Created().finish()
}
async fn get_apps_for_admin(
    connection: web::Data<PgPool>,
    app_id: web::Path<i32>,
    req: HttpRequest,
) -> impl Responder {
    let Some(user) = req.extensions().get::<BiscuitInfo>().map(|b| {b.user_id}) else {
        return HttpResponse::Unauthorized().finish();
    };
    let app = AppId(*app_id);

    if let Ok(apps) = app.get_all_for_user(user, &connection).await {
        HttpResponse::Ok().json(apps)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

async fn delete_app(
    connection: web::Data<PgPool>,
    app_id: web::Path<i32>,
    req: HttpRequest,
) -> impl Responder {
    let Some(user) = req.extensions().get::<BiscuitInfo>().map(|b| {b.user_id}) else {
        return HttpResponse::Unauthorized().finish();
    };
    let app = AppId(*app_id);

    let Ok(apps) = app.get_all_for_user(user, &connection).await
    else {
        return HttpResponse::InternalServerError().finish();
    };
    if apps
        .iter()
        .find(|a| a.app_id == app)
        .map(|a| true)
        .unwrap_or(false)
    {
        app.delete(&connection).await.unwrap();
    }
    return HttpResponse::Unauthorized().finish();
}
