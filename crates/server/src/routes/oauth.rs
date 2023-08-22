use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;

use serde::Serialize;
use sqlx::PgPool;

use shared::BiscuitInfo;

use crate::auth_user::validator;
use crate::models::user::UserId;

pub(crate) fn routes() -> impl HttpServiceFactory {
    web::scope("api/v1/oauth/whoami")
        .wrap(HttpAuthentication::bearer(validator))
        .route("", web::get().to(whoami))
}

#[derive(Serialize)]
struct Identity {
    user_id: UserId,
    name: String,
}

async fn whoami(
    account: web::ReqData<BiscuitInfo>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    let user_id = UserId::from(account.user_id);
    HttpResponse::Ok().json(Identity {
        user_id,
        name: user_id.get(&connection).await.unwrap().name,
    })
}
