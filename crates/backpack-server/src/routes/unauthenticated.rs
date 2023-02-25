use actix_web::{dev::HttpServiceFactory, web};
use biscuit_auth::KeyPair;

pub mod email_password;
pub mod health_check;

pub fn config(kp: web::Data<KeyPair>) -> impl HttpServiceFactory {
    web::scope("/unauthenticated")
        .service(email_password::config(kp))
        .service(health_check::config())
}
