use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;

use serde::Serialize;
use sqlx::PgPool;

use crate::auth_user::{validator, BiscuitInfo};

use crate::models::user::UserId;

pub(crate) fn routes() -> impl HttpServiceFactory {
    web::scope("oauth/whoami")
        .wrap(HttpAuthentication::bearer(validator))
        .route("", web::get().to(whoami))
}

#[derive(Serialize)]
struct Identity<'a> {
    user_id: &'a UserId,
    name: String,
}

async fn whoami(
    account: web::ReqData<BiscuitInfo>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    HttpResponse::Ok().json(Identity {
        user_id: &account.user_id,
        name: account.user_id.get(&connection).await.unwrap().name,
    })
}
