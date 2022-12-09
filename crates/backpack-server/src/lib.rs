use actix_web::{
    dev::Server,
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use configuration::get_configuration;
use sqlx::PgPool;
use std::net::TcpListener;

pub mod configuration;
pub mod domains;
pub mod random_names;

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let config = Data::new(get_configuration());
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
            //.service(domains::admin::admin(root.clone()))
            //.service(domains::leaderboard::leaderboard(root.clone()))
            .service(domains::user::user())
            .service(domains::item::item())
            .service(domains::user_item::user_item())
        //.route("/{filename:.*}", web::get().to(spa))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
