use actix_cors::Cors;
use actix_web::{dev::HttpServiceFactory, web, HttpResponse, Responder, Scope};
use biscuit_auth::KeyPair;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    auth_user::Role,
    biscuit::TokenReply,
    configuration::Settings,
    models::email_password::{create, exist, find},
    random_names::random_name,
};
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::models::user::UserId;

pub(crate) fn oauth_email_password(kp: web::Data<KeyPair>) -> impl HttpServiceFactory {
    let cors = Cors::default()
        .allow_any_header()
        .allow_any_origin()
        .allow_any_method()
        .send_wildcard()
        .max_age(3600);
    web::scope("api/v1/auth/email_password")
        .app_data(kp)
        .wrap(cors)
        .route("create", web::post().to(oauth_create_email_password))
        .route("login", web::post().to(oauth_login_email_password))
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateEmailPasswordData {
    pub email: String,
}

async fn oauth_create_email_password(
    req_data: web::Json<CreateEmailPasswordData>,
    connection: web::Data<PgPool>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct LoginEmailPasswordData {
    pub email: String,
    pub password_plain: String,
}

async fn oauth_login_email_password(
    req_data: web::Json<LoginEmailPasswordData>,
    connection: web::Data<PgPool>,
    root: web::Data<KeyPair>,
) -> impl Responder {
    let Ok((email_password_id, password_hash_existing, user_id)) =
        find(connection.as_ref(), &req_data.email).await else {
            // We do not return not found, to avoid giving information about an account existance.
            return HttpResponse::Unauthorized().finish();
        };
    let Ok(true) =
        verify(dbg!(&req_data.password_plain), &password_hash_existing)
        else {
        return HttpResponse::Unauthorized().finish();
    };
    // TODO: set email password as verified ? (or create another route to do that, it would probably be better.)
    let biscuit = user_id.create_biscuit(&root, Role::Admin);
    HttpResponse::Ok().json(dbg!(TokenReply {
        token: biscuit.to_base64().unwrap(),
    }))
}
