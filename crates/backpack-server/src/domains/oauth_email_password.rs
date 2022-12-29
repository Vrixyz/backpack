use std::collections::HashMap;

use actix_web::{web, HttpResponse, Responder, Scope};
use biscuit_auth::KeyPair;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    auth_user::Role,
    configuration::Settings,
    domains::{oauth::TokenReply, user::UserId, user_github::GithubUser},
    random_names::random_name,
};

#[derive(Debug, Deserialize)]
pub struct OauthCode {
    email: String,
}

#[derive(Deserialize)]
pub struct GithubOauthResponse {
    access_token: String,
}

async fn oauth_create_email_password(
    code: web::Query<OauthCode>,
    config: web::Data<Settings>,
    connection: web::Data<PgPool>,
    root: web::Data<KeyPair>,
) -> impl Responder {
    // TODO: check if there is no email_password created with passed login

    // wrong code:
    /*
    let user = if gh_user.exist(&connection).await {
        gh_user.get_user(&connection).await.unwrap()
    } else {
        let user = UserId::create(&connection, &random_name()).await.unwrap();
        assert!(gh_user.create(&connection, user).await);
        user
    };*/

    // TODO: send mail with password
    /*
        let email = Message::builder()
            .from(
                format!("{}", dotenv::var("BACKPACK_EMAIL").unwrap())
                    .parse()
                    .unwrap(),
            )
            .reply_to(dotenv::var("BACKPACK_EMAIL").unwrap().parse().unwrap())
            .to("the_email_to_send_to".parse().unwrap())
            .subject("Test rust")
            .body(String::from("mail from Rust!"))
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
    */

    // FIXME: we should not create biscuit here,
    // there should be another route where user provides the received password
    // then later we can flag the user as verified ? :shrug:

    HttpResponse::Ok()
}

pub(crate) fn oauth_github() -> Scope {
    web::scope("oauth/email_password").route("create", web::post().to(oauth_create_email_password))
}
