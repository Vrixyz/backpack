use actix_web::{dev::HttpServiceFactory, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;

use crate::auth_user::validator_admin;

mod app;
mod item;

pub fn config(kp: web::Data<KeyPair>) -> impl HttpServiceFactory {
    web::scope("/admin")
        .app_data(kp)
        .wrap(HttpAuthentication::bearer(validator_admin))
        .service(app::config())
        .service(item::config())
}
