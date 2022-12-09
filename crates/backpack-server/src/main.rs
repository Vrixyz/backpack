use backpack_server::run;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/backpack")
        .await
        .expect("Failed to connect to Postgres.");

    let address = "0.0.0.0:8080";
    let listener = TcpListener::bind(address)?;

    run(listener, pool)?.await
}
