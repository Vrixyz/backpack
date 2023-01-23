use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use reqwest::Method;

pub fn config() -> impl HttpServiceFactory {
    web::resource("/health_check").route(web::get().to(health_check))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}
