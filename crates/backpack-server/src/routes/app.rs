use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web::{HttpMessage, HttpRequest};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;
use serde::Deserialize;
use sqlx::PgPool;

use crate::auth_user::{validator_admin, BiscuitInfo};

use crate::models::app::AppAdmin;
use crate::models::{app::AppId, user::UserId};

pub(crate) fn app_admin(kp: web::Data<KeyPair>) -> impl HttpServiceFactory {
    let cors = Cors::default()
        .allow_any_header()
        .allow_any_origin()
        .allow_any_method()
        .send_wildcard()
        .max_age(3600);
    web::scope("api/v1")
        .app_data(kp)
        // FIXME: this lines makes it fail, read the doc...
        .wrap(HttpAuthentication::bearer(validator_admin))
        .wrap(cors)
        //
        .route("app", web::post().to(create_app))
        .route("app", web::get().to(get_apps_for_admin))
        .route("app", web::delete().to(delete_app))
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateAppData {
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
pub struct DeleteAppData {
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
    if apps
        .iter()
        .find(|a| a.app_id == app)
        .map(|a| true)
        .unwrap_or(false)
    {
        app.delete(&connection).await.unwrap();
        return HttpResponse::Ok().finish();
    }
    return HttpResponse::Unauthorized().finish();
}
