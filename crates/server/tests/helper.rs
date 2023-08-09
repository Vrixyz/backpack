use std::net::TcpListener;

use backpack_client::{
    shared::{AppId, BiscuitInfo, Role, UserId},
    BackpackClient, RequestError,
};
use backpack_server::{
    configuration::{get_configuration, DatabaseSettings, Settings},
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub db_pool: PgPool,
    pub api_client: BackpackClient,
    pub settings: Settings,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let mut settings = get_configuration();
    settings.database.database_name = format!("test-{}", Uuid::new_v4());
    settings.application_port = port;
    let connection_pool = configure_database(&settings.database).await;

    let server = backpack_server::run(listener, connection_pool.clone(), settings.clone())
        .expect("Failed to bind address");
    drop(tokio::spawn(server));

    let url = format!("http://127.0.0.1:{port}");
    TestApp {
        db_pool: connection_pool,
        api_client: BackpackClient::new(url + "/api/v1"),
        settings,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub struct TestUser {
    user_id: UserId,
    pub email: String,
    pub password: String,
}
#[derive(Debug)]
pub struct UserAuthentication {
    pub biscuit_raw: Vec<u8>,
    pub infos: BiscuitInfo,
}

impl TestUser {
    pub async fn generate(client: &mut BackpackClient) -> Result<Self, RequestError> {
        let email = Uuid::new_v4().to_string() + "@example.com";
        let created_data = client
            .signup(&backpack_client::shared::CreateEmailPasswordData {
                email: email.clone(),
            })
            .await?;

        Ok(Self {
            user_id: created_data.id,
            email,
            password: created_data.password,
        })
    }
    pub async fn login(
        &self,
        client: &mut BackpackClient,
        as_app_user: Option<AppId>,
    ) -> Result<UserAuthentication, RequestError> {
        let biscuit = client
            .login(&backpack_client::shared::LoginEmailPasswordData {
                email: self.email.clone(),
                password_plain: self.password.clone(),
                as_app_user,
            })
            .await?;

        assert!(
            biscuit.1.role
                == match as_app_user {
                    Some(app_id) => Role::User(app_id),
                    None => Role::Admin,
                }
        );

        Ok(UserAuthentication {
            biscuit_raw: biscuit.0,
            infos: biscuit.1,
        })
    }
}
