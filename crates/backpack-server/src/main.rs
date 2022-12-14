use backpack_server::{configuration::get_configuration, run};
use dotenv::dotenv;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let configuration = get_configuration();

    let address = format!(
        "{}:{}",
        configuration.application_host, configuration.application_port
    );
    dbg!(&address);

    let listener = TcpListener::bind(&address)?;
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    run(listener, connection_pool, configuration)?.await
}
