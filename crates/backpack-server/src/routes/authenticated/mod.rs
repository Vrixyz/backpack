use actix_web::{dev::HttpServiceFactory, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;

use crate::auth_user::validator;

mod user_item;

pub fn config(kp: web::Data<KeyPair>) -> impl HttpServiceFactory {
    web::scope("/authenticated")
        .app_data(kp)
        .wrap(HttpAuthentication::bearer(validator))
        .service(user_item::config())
}
