use std::net::TcpListener;

use backpack_client::BackpackClient;
use backpack_server::{
    configuration::{get_configuration, DatabaseSettings},
    models::user::UserId,
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
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = get_configuration();
    configuration.database.database_name = format!("test-{}", Uuid::new_v4());
    configuration.application_port = port;
    let connection_pool = configure_database(&configuration.database).await;

    let server = backpack_server::run(listener, connection_pool.clone(), configuration)
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);

    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();
    let url = format!("http://127.0.0.1:{port}");
    TestApp {
        db_pool: connection_pool,
        api_client: BackpackClient::new_with_client(url + "/api/v1", api_client),
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

impl TestUser {
    pub async fn generate(client: &mut BackpackClient) -> Result<Self, reqwest::Error> {
        let email = Uuid::new_v4().to_string() + "@example.com";
        // TODO: send signup and retrieve password and id.
        let password = client
            .signup(&backpack_client::shared::CreateEmailPasswordData {
                email: email.clone(),
            })
            .await?;
        todo!("Get password and user id from json.");
        /* Json is: (see CreatedUserEmailPasswordData, return that from within client.signup)
        {
            "id": 2,
            "password": "XFbUnzBs~WP)y8u*"
          }
        */
        let user_id = UserId(0);

        Ok(Self {
            user_id,
            email,
            password,
        })
    }
    // TODO: login, etc.
}
