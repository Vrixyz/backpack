use std::collections::HashMap;

use actix_web::{web, HttpResponse, Responder, Scope};
use biscuit_auth::KeyPair;
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
    code: String,
}

async fn oauth_fake_success(
    config: web::Data<Settings>,
    connection: web::Data<PgPool>,
    root: web::Data<KeyPair>,
) -> impl Responder {
    let mut params = HashMap::new();
    params.insert("client_id", &config.github_admin_app.client_id);
    params.insert("client_secret", &config.github_admin_app.client_secret);

    let gh_user = GithubUser {
        login: "fake".into(),
        id: 0,
    };

    let user = if gh_user.exist(&connection).await {
        gh_user.get_user(&connection).await.unwrap()
    } else {
        let user = UserId::create(&connection, &random_name()).await.unwrap();
        dbg!(user);
        // FIXME: this fails when run with tests

        assert!(gh_user.create(&connection, user).await);
        user
    };

    let biscuit = user.create_biscuit(&root, Role::Admin);
    dbg!(HttpResponse::Ok().json(TokenReply {
        token: biscuit.to_base64().unwrap(),
    }))
}

#[cfg(debug_assertions)]
pub(crate) fn oauth_fake() -> Scope {
    web::scope("oauth/fake").route("success", web::get().to(oauth_fake_success))
}
#[cfg(not(debug_assertions))]
pub(crate) fn oauth_fake() -> Scope {
    web::scope("oauth").route("success", web::get().to(oauth_fake_success))
    //   web::scope("api/v1/oauth/").route("success", web::get().to(|_| {}))
}
