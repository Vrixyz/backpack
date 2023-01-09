use std::collections::HashMap;

use actix_web::{web, HttpResponse, Responder, Scope};
use biscuit_auth::KeyPair;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    auth_user::Role,
    biscuit::TokenReply,
    configuration::Settings,
    models::{user::UserId, user_github::GithubUser},
    random_names::random_name,
};

#[derive(Debug, Deserialize)]
pub struct OauthCode {
    code: String,
}

#[derive(Deserialize)]
pub struct GithubOauthResponse {
    access_token: String,
}

async fn oauth_callback(
    code: web::Query<OauthCode>,
    config: web::Data<Settings>,
    connection: web::Data<PgPool>,
    root: web::Data<KeyPair>,
) -> impl Responder {
    let mut params = HashMap::new();
    params.insert("client_id", &config.github_admin_app.client_id);
    params.insert("client_secret", &config.github_admin_app.client_secret);
    params.insert("code", &code.code);

    let client = reqwest::Client::new();

    let response = client
        .post("https://github.com/login/oauth/access_token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap();

    let github_bearer = response
        .json::<GithubOauthResponse>()
        .await
        .unwrap()
        .access_token;
    let gh_user = client
        .get("https://api.github.com/user")
        .bearer_auth(github_bearer)
        .header("user-agent", "backpack")
        .send()
        .await
        .unwrap()
        .json::<GithubUser>()
        .await
        .unwrap();

    let user = if gh_user.exist(&connection).await {
        gh_user.get_user(&connection).await.unwrap()
    } else {
        let user = UserId::create(&connection, &random_name()).await.unwrap();
        assert!(gh_user.create(&connection, user).await);
        user
    };

    let biscuit = user.create_biscuit(&root, Role::Admin);
    HttpResponse::Ok().json(TokenReply {
        token: biscuit.to_base64().unwrap(),
    })
}
pub(crate) fn oauth_github() -> Scope {
    web::scope("oauth/github").route("callback", web::get().to(oauth_callback))
}
