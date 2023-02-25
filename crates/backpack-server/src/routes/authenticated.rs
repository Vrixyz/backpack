use actix_web::{dev::HttpServiceFactory, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;

use crate::auth_user::validator;

mod app;
mod item;
mod user;
mod whoami;

pub fn config(kp: web::Data<KeyPair>) -> impl HttpServiceFactory {
    web::scope("/authenticated")
        .app_data(kp)
        .wrap(HttpAuthentication::bearer(validator))
        .service(item::config())
        .service(app::config())
        .service(whoami::config())
        .service(user::config())
}
