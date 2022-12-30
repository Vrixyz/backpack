use actix_web::{web, HttpResponse, Responder, Scope};
use biscuit_auth::KeyPair;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{configuration::Settings, random_names::random_name};
use bcrypt::{hash, verify, DEFAULT_COST};

use super::user::UserId;

/// Checks if a email exists as an email/password record.
pub async fn exist(connection: &PgPool, email: &str) -> bool {
    sqlx::query!(
        "SELECT id FROM users_email_password WHERE email = $1",
        email
    )
    .fetch_one(connection)
    .await
    .is_ok()
}
/// Meant to be used with another query following, to link it to this authentication method.
/// FIXME: This API could be reworked to be misuse resistant.
pub async fn create(
    connection: &PgPool,
    email: &str,
    password_hash: &str,
    account: UserId,
) -> bool {
    sqlx::query!(
        r#"
            INSERT INTO users_email_password (email, password_hash, is_verified, user_id) VALUES ($1, $2, $3, $4)
            RETURNING id
        "#,
        email,
        password_hash,
        false,
        *account,
    )
    .fetch_one(connection)
    .await
    .map(|_| true)
    .is_ok()
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateEmailPasswordData {
    pub email: String,
}

async fn oauth_create_email_password(
    req_data: web::ReqData<CreateEmailPasswordData>,
    config: web::Data<Settings>,
    connection: web::Data<PgPool>,
    root: web::Data<KeyPair>,
) -> impl Responder {
    use rand::Rng;
    if exist(connection.as_ref(), &req_data.email).await {
        // User should use login with email password.
        return HttpResponse::ExpectationFailed();
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
        return HttpResponse::InternalServerError();
    };

    let user = UserId::create(&connection, &random_name()).await.unwrap();
    assert!(create(&connection, &req_data.email, &password_hashed, user).await);

    let email = Message::builder()
        .from(
            format!("{}", dotenv::var("BACKPACK_EMAIL").unwrap())
                .parse()
                .unwrap(),
        )
        .reply_to(dotenv::var("BACKPACK_EMAIL").unwrap().parse().unwrap())
        .to(req_data.email.parse().unwrap())
        .subject("Welcome to Backpack")
        .body(format!(
            "Hi,\nWelcome to Backpack, your password is {}.",
            &password
        ))
        .unwrap();
    dbg!(&email);
    let creds = Credentials::new(
        dotenv::var("BACKPACK_EMAIL").unwrap().to_string(),
        dotenv::var("BACKPACK_EMAIL_PASSWORD").unwrap().to_string(),
    );

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.zoho.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }

    // We should not create biscuit here, because we need to verify the email first.
    // there should be another route where user provides the received password along with the email.
    // THOUGHTS: it could be a direct link from the email.
    // - my first idea was to let the password in the email, and ask the user to ente it on backpack login page.
    // - I'm tempted to share a link like "/login?email=bla&password=password"
    // But sharing password in query url is a bad practice,
    // but I guess it could be fine as a one time password though,
    // and force password regeneration, and return it to the user in plain text, so he can save it.
    // - Or, we only allow connection via a one time link through email, and invalidate password after the connection.
    // When the user clicks on a link, he is redirected to login,
    // gets an authentication token and can use services until its expiration.
    // then later we can flag the user as verified ? :shrug:

    HttpResponse::Created()
}

pub(crate) fn oauth_email_password() -> Scope {
    web::scope("oauth/email_password").route("create", web::post().to(oauth_create_email_password))
}
