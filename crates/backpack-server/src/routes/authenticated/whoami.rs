use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;

use crate::auth_user::BiscuitInfo;
use crate::models::user::UserId;

pub(crate) fn config() -> impl HttpServiceFactory {
    web::resource("/whoami").route(web::get().to(whoami))
}

async fn whoami(account: web::ReqData<BiscuitInfo>) -> impl Responder {
    HttpResponse::Ok().json(&*account)
}
