use std::collections::HashMap;

use actix_web::{
    cookie::time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime},
    web, HttpResponse, Responder, Scope,
};
use biscuit_auth::{
    builder::{Fact, Term},
    Authorizer, Biscuit, KeyPair,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{configuration::Settings, random_names::random_name};

use super::{
    user::{User, UserId},
    user_github::GithubUser,
};

pub const TOKEN_TTL: i64 = 600;

#[derive(Serialize, Deserialize)]
pub struct TokenReply {
    pub token: String,
}

trait BiscuitFact: Sized {
    fn as_biscuit_fact(&self) -> Fact;
    fn from_authorizer(authorizer: &mut Authorizer) -> Option<Self>;
}

impl BiscuitFact for UserId {
    fn as_biscuit_fact(&self) -> Fact {
        Fact::new("user".to_string(), vec![Term::Str((*self).to_string())])
    }

    fn from_authorizer(authorizer: &mut Authorizer) -> Option<Self> {
        let res: Vec<(String,)> = authorizer.query("data($id) <- user($id)").ok()?;
        Some(UserId(res.get(0)?.0.as_str().parse::<i32>().ok()?))
    }
}

impl UserId {
    pub fn create_biscuit(&self, root: &KeyPair) -> Biscuit {
        let mut builder = Biscuit::builder(root);
        builder.add_authority_fact(self.as_biscuit_fact()).unwrap();

        builder
            .add_authority_check(
                format!(
                    r#"check if time($time), $time < {}"#,
                    (OffsetDateTime::now_utc() + Duration::seconds(TOKEN_TTL))
                        .format(&Rfc3339)
                        .unwrap()
                )
                .as_str(),
            )
            .unwrap();

        builder.build().unwrap()
    }
}

pub fn authorize(token: &Biscuit) -> Option<UserId> {
    let mut authorizer = token.authorizer().ok()?;

    authorizer.set_time();
    authorizer.allow().map_err(|_| ()).ok()?;
    authorizer.authorize().map_err(|_| ()).ok()?;

    UserId::from_authorizer(&mut authorizer)
}

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

    dbg!(&params);
    let response = dbg!(client
        .post("https://github.com/login/oauth/access_token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap());
    //dbg!(response.text().await);

    let github_bearer = response
        .json::<GithubOauthResponse>()
        .await
        .unwrap()
        .access_token;
    dbg!(&github_bearer);
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

    let biscuit = user.create_biscuit(&root);
    HttpResponse::Ok().json(TokenReply {
        token: biscuit.to_base64().unwrap(),
    })
}

pub(crate) fn oauth() -> Scope {
    web::scope("oauth").route("callback", web::get().to(oauth_callback))
}

#[derive(Serialize)]
struct Identity<'a> {
    admin: &'a UserId,
    github: Option<GithubUser>,
}
