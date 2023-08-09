use actix_web::{dev::HttpServiceFactory, web};
use biscuit_auth::KeyPair;

use crate::time::MockableDateTime;

pub mod email_password;
pub mod health_check;

pub fn config(
    kp: web::Data<KeyPair>,
    time: web::Data<MockableDateTime>,
) -> impl HttpServiceFactory {
    web::scope("/unauthenticated")
        .service(email_password::config(kp, time))
        .service(health_check::config())
}
