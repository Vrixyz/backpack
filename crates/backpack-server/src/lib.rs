use actix_web::{
    dev::Server,
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use configuration::Settings;
use sqlx::PgPool;
use std::net::TcpListener;

pub mod auth_user;
pub mod configuration;
pub mod domains;
pub mod random_names;

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    settings: Settings,
) -> Result<Server, std::io::Error> {
    let config = Data::new(settings);
    let root = Data::new(config.get_keypair());
    let connection = Data::new(connection_pool);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(connection.clone())
            .app_data(root.clone())
            .app_data(config.clone())
            .wrap(Logger::default())
            .route(
                "/health_check",
                web::get().to(domains::healthcheck::health_check),
            )
            //.service(domains::config::config(config.clone()))
            .service(domains::oauth::oauth())
            .service(domains::app_admin::app_admin(root.clone()))
            .service(domains::item::item())
            .service(domains::user_item::user_item())
        //.route("/{filename:.*}", web::get().to(spa))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
