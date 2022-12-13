use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use biscuit_auth::KeyPair;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth_user::validator;

use super::{app::AppId, user::UserId};

#[derive(Serialize, Deserialize)]
pub struct AppAdmin {
    pub user_id: UserId,
    pub app_id: AppId,
}

impl AppAdmin {
    async fn create_app_admin_relation(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let _rec = sqlx::query!(
            r#"
    INSERT INTO apps_admins ( user_id, app_id )
    VALUES ( $1, $2)
            "#,
            *self.user_id,
            *self.app_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

pub(crate) fn app_admin(kp: web::Data<KeyPair>) -> impl HttpServiceFactory {
    web::scope("api/v1")
        .app_data(kp)
        .wrap(HttpAuthentication::bearer(validator))
        .route("app", web::post().to(create_app))
}

/// FIXME: This should do app creation and admin association in a single request.
async fn create_app(
    user_id: web::ReqData<UserId>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    let app_id = AppId::create(&connection, "Placeholder app").await.unwrap();
    AppAdmin {
        user_id: *user_id,
        app_id,
    }
    .create_app_admin_relation(&connection)
    .await
    .unwrap();
    HttpResponse::Created()
}
