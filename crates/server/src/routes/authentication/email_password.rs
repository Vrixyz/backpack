use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder};
use biscuit_auth::KeyPair;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use rand::Rng;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    models::{
        app::AppId,
        email_password::{create, exist, find},
    },
    random_names::random_name,
    time::MockableDateTime,
};
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::models::user::UserId;

use super::auth::create_new_authentication_token;

pub fn config(
    kp: web::Data<KeyPair>,
    time: web::Data<MockableDateTime>,
) -> impl HttpServiceFactory {
    web::scope("/email_password")
        .app_data(time)
        .app_data(kp)
        .route("create", web::post().to(create_email_password))
        .route("login", web::post().to(login_email_password))
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateEmailPasswordData {
    pub email: String,
}
impl std::fmt::Display for CreateEmailPasswordData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.email)
    }
}

fn expose_secret_string<S>(data: &Secret<String>, ser: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    ser.serialize_str(data.expose_secret())
}

#[derive(Debug, Serialize, Clone)]
pub struct CreatedUserEmailPasswordData {
    pub id: UserId,
    #[serde(serialize_with = "expose_secret_string")]
    pub password: Secret<String>,
}

#[tracing::instrument(
    name = "create_email_password",
    skip_all,
    fields(req_data=%&*req_data)
)]
async fn create_email_password(
    connection: web::Data<PgPool>,
    req_data: web::Json<CreateEmailPasswordData>,
) -> impl Responder {
    use rand::Rng;
    if exist(connection.as_ref(), &req_data.email).await {
        // User should use login with email password.
        return HttpResponse::ExpectationFailed().finish();
    }
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 16;
    let mut rng = rand::thread_rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    let Ok(password_hashed) = hash(&password, DEFAULT_COST) else {
        return HttpResponse::InternalServerError().finish();
    };

    let user = UserId::create(&connection, &random_name()).await.unwrap();
    assert!(create(&connection, &req_data.email, &password_hashed, user).await);

    let email = Message::builder()
        .from(dotenvy::var("BACKPACK_EMAIL").unwrap().parse().unwrap())
        .reply_to(dotenvy::var("BACKPACK_EMAIL").unwrap().parse().unwrap())
        .to(req_data.email.parse().unwrap())
        .subject("Welcome to Backpack")
        .body(format!(
            "Hi,\nWelcome to Backpack, your password is {password}.",
        ))
        .unwrap();
    let creds = Credentials::new(
        dotenvy::var("BACKPACK_EMAIL").unwrap(),
        dotenvy::var("BACKPACK_EMAIL_PASSWORD").unwrap(),
    );

    // Open a remote connection to mail service
    let mailer = SmtpTransport::relay("smtp.zoho.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => {}
        Err(e) => panic!("Could not send email: {e:?}"),
    }

    // We should not create biscuit here, because we need to verify the email first.
    // there should be another route where user provides the received password along with the email.
    // THOUGHTS: it could be a direct link from the email.
    // - my first idea was to let the password in the email, and ask the user to enter it on backpack login page.
    // - I'm tempted to share a link like "/login?email=bla&password=password"
    // But sharing password in query url is a bad practice,
    // but I guess it could be fine as a one time password though,
    // and force password regeneration, and return it to the user in plain text, so he can save it.
    // - Or, we only allow connection via a one time link through email, and invalidate password after the connection.
    // When the user clicks on a link, he is redirected to login,
    // gets an authentication token and can use services until its expiration.
    // then later we can flag the user as verified ? :shrug:

    HttpResponse::Created().json(CreatedUserEmailPasswordData {
        id: user,
        password: Secret::new(password),
    })
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoginEmailPasswordData {
    pub email: String,
    pub password_plain: String,
    pub as_app_user: Option<AppId>,
}

#[tracing::instrument(name = "login_email_password", skip_all)]
async fn login_email_password(
    req_data: web::Json<LoginEmailPasswordData>,
    connection: web::Data<PgPool>,
    root: web::Data<KeyPair>,
    time: web::Data<MockableDateTime>,
) -> HttpResponse {
    let Ok((_email_password_id, password_hash_existing, user_id)) =
        find(connection.as_ref(), &req_data.email).await else {
            // We do not return not found, to avoid giving information about an account existence.
            return dbg!(HttpResponse::Unauthorized().finish());
        };
    let Ok(true) =
        verify(&req_data.password_plain, &password_hash_existing)
        else {
        return dbg!(HttpResponse::Unauthorized().finish());
    };
    // TODO: set email password as verified ? (or create another route to do that, it would probably be better.)
    create_new_authentication_token(connection, root, time, user_id, req_data.as_app_user).await
}
