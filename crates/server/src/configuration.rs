use base64::{engine::general_purpose, Engine as _};
use biscuit_auth::{KeyPair, PrivateKey};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_host: String,
    pub application_port: u16,
    pub private_key: Option<String>,
    pub github_admin_app: OAuth,
}

#[derive(Deserialize, Debug)]
pub struct OAuth {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

pub fn get_configuration() -> Settings {
    let file = std::env::var("CONFIGURATION").unwrap_or_else(|_| "configuration.dhall".to_string());
    serde_dhall::from_file(file).parse().unwrap()
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

impl Settings {
    pub fn get_keypair(&self) -> KeyPair {
        self.private_key
            .as_ref()
            .and_then(|pk_string| general_purpose::STANDARD.decode(pk_string).ok())
            .and_then(|pk_bytes| PrivateKey::from_bytes(&pk_bytes).ok())
            .map(KeyPair::from)
            .unwrap_or_else(|| {
                dbg!("Creating new private key.");
                KeyPair::new()
                // What you should store into a file, or env:
                // dbg!(general_purpose::STANDARD.encode(key.private().to_bytes()));
            })
    }
}
