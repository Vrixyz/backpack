use actix_web::{dev::HttpServiceFactory, web};
use biscuit_auth::KeyPair;

use crate::time::MockableDateTime;

pub mod auth;
pub mod email_password;
pub mod health_check;

pub fn config(
    kp: web::Data<KeyPair>,
    time: web::Data<MockableDateTime>,
) -> impl HttpServiceFactory {
    web::scope("/authentication")
        .service(auth::config(kp.clone(), time.clone()))
        .service(email_password::config(kp, time))
        .service(health_check::config())
}
