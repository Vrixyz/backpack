use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};

use shared::BiscuitInfo;

pub(crate) fn config() -> impl HttpServiceFactory {
    web::resource("/whoami").route(web::get().to(whoami))
}

#[tracing::instrument(
    name = "Get user",
    skip_all,
    fields(account=%&*account)
)]
async fn whoami(account: web::ReqData<BiscuitInfo>) -> impl Responder {
    HttpResponse::Ok().json(&*account)
}
