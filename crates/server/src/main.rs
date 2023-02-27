use backpack_server::{
    configuration::get_configuration,
    run,
    telemetry::{get_subscriber, init_subscriber},
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let configuration = get_configuration();

    let subscriber = get_subscriber("backpack".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let address = format!(
        "{}:{}",
        configuration.application_host, configuration.application_port
    );
    dbg!(&address);

    let listener = TcpListener::bind(&address)?;
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    run(listener, connection_pool, configuration)?.await
}
