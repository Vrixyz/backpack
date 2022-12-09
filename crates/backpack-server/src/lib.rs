use actix_web::{dev::Server, middleware::Logger, web::Data, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

pub mod domains;

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let connection = Data::new(connection_pool);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(connection.clone())
            .wrap(Logger::default())
            /*            .route(
                "/health_check",
                web::get().to(domains::healthcheck::health_check),
            )*/
            //.service(domains::config::config(config.clone()))
            //.service(domains::oauth::oauth())
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
