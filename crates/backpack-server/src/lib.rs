use actix_cors::Cors;
use actix_web::{
    dev::Server,
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use configuration::{DatabaseSettings, Settings};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;

pub mod auth_user;
pub mod biscuit;
pub mod configuration;
pub mod models;
pub mod random_names;
pub mod routes;
pub mod telemetry;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    settings: Settings,
) -> Result<Server, std::io::Error> {
    let config = Data::new(settings);
    let root = Data::new(config.get_keypair());
    let connection = Data::new(connection_pool);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_origin()
            .allow_any_method()
            .send_wildcard()
            .max_age(3600);
        App::new()
            .app_data(connection.clone())
            .app_data(config.clone())
            .app_data(root.clone())
            .wrap(Logger::default())
            .wrap(cors)
            .service(
                web::scope("/api/v1")
                    .service(routes::admin::config(root.clone()))
                    .service(routes::authenticated::config(root.clone()))
                    .service(routes::unauthenticated::config(root.clone())),
            )
            //
            //
            // WIP
            //
            //
            //.service(domains::config::config(config.clone()))
            .service(models::oauth_github::oauth_github())
            .service(routes::oauth::routes())

        //
        //

        //.route("/{filename:.*}", web::get().to(spa))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
