use std::{borrow::BorrowMut, collections::HashMap, ops::DerefMut};

use actix_web::{
    cookie::time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime},
    web, HttpResponse, Responder, Scope,
};
use biscuit_auth::{
    builder::{BiscuitBuilder, Fact, Term},
    error, Authorizer, Biscuit, KeyPair,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    auth_user::{BiscuitInfo, Role},
    configuration::Settings,
    random_names::random_name,
};

use super::{
    app::AppId,
    user::{User, UserId},
    user_github::GithubUser,
};

pub const TOKEN_TTL: i64 = 600;

#[derive(Serialize, Deserialize)]
pub struct TokenReply {
    pub token: String,
}

pub trait BiscuitBaker<'a, T> {
    fn bake(&'a mut self, ingredient: T) -> &'a mut BiscuitBuilder<'a>;
}

impl<'a> TryFrom<&'a mut Authorizer<'a>> for Role {
    type Error = String;

    fn try_from(authorizer: &mut Authorizer) -> Result<Self, Self::Error> {
        let admin: Option<Vec<(bool,)>> =
            authorizer.query("data($is_admin) <- role($is_admin)").ok();
        match admin {
            Some(res) if res.len() > 0 && res[0].0 => Ok(Role::Admin),
            _ => {
                let app_id: Vec<(String,)> = authorizer
                    .query("data($app_id) <- user_app_id($app_id)")
                    .map_err(|_| "query app_id error")?;
                Ok(Role::User(AppId(
                    app_id
                        .get(0)
                        .ok_or("get(0) error")?
                        .0
                        .as_str()
                        .parse::<i32>()
                        .map_err(|_| "parse error")?,
                )))
            }
        }
    }
}

impl<'a> TryFrom<&'a mut Authorizer<'a>> for UserId {
    type Error = String;

    fn try_from(value: &mut Authorizer) -> Result<Self, Self::Error> {
        let res: Vec<(String,)> = value
            .query("data($id) <- role($id)")
            .map_err(|_| "query error")?;
        Ok(UserId(
            res.get(0)
                .ok_or("get(0) error")?
                .0
                .as_str()
                .parse::<i32>()
                .map_err(|_| "parse error")?,
        ))
    }
}
impl<'a, 'b: 'a> TryFrom<&'b mut Authorizer<'a>> for BiscuitInfo {
    type Error = String;

    fn try_from(authorizer: &'a mut Authorizer<'b>) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: UserId::try_from(&mut authorizer.clone())?,
            role: Role::try_from(authorizer)?,
        })
    }
}

impl<'a, 'b: 'a> BiscuitBaker<'b, UserId> for BiscuitBuilder<'a> {
    fn bake(&'a mut self, ingredient: UserId) -> &'a mut BiscuitBuilder<'a> {
        self.add_authority_fact(Fact::new(
            "user".to_string(),
            vec![Term::Str((*ingredient).to_string())],
        ));
        self
    }
}

impl<'a> BiscuitBaker<'a, Role> for BiscuitBuilder<'a> {
    fn bake(&'a mut self, ingredient: Role) -> &'a mut BiscuitBuilder<'a> {
        match ingredient {
            Role::Admin => {
                self.add_authority_fact(Fact::new("admin".to_string(), vec![Term::Bool(true)]));
            }
            Role::User(app_id) => {
                self.add_authority_fact(Fact::new(
                    "user_app_id".to_string(),
                    vec![Term::Str((app_id).to_string())],
                ));
            }
        }
        self
    }
}

impl UserId {
    pub fn create_biscuit(&self, root: &KeyPair) -> Biscuit {
        let mut builder = Biscuit::builder(root);

        // FIXME: this should be better in a function but borrowing said no :(
        {
            let ref mut this = builder;
            let ingredient = *self;
            this.add_authority_fact(Fact::new(
                "user".to_string(),
                vec![Term::Str((*ingredient).to_string())],
            ));
            this
        }
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

pub fn authorize_user_only(token: &Biscuit) -> Option<UserId> {
    let mut authorizer = token.authorizer().ok()?;

    authorizer.set_time();
    authorizer.allow().map_err(|_| ()).ok()?;
    authorizer.authorize().map_err(|_| ()).ok()?;

    UserId::try_from(&mut authorizer)
        .map_err(|_| "authorize error")
        .ok()
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
    web::scope("oauth")
        .route("callback", web::get().to(oauth_callback))
        .route("whoami", web::get().to(whoami))
}

#[derive(Serialize)]
struct Identity<'a> {
    user_id: &'a UserId,
    name: String,
}

async fn whoami(account: web::ReqData<UserId>, connection: web::Data<PgPool>) -> impl Responder {
    HttpResponse::Ok().json(Identity {
        user_id: &account,
        name: account.get(&connection).await.unwrap().name,
    })
}
