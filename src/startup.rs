use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

use crate::routes::{health_check, subscribe};

pub fn run_server(
    tcp_listener: TcpListener,
    db_conn_pool: PgPool,
) -> Result<Server, std::io::Error> {
    let db_conn_pool = web::Data::new(db_conn_pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_conn_pool.clone())
    })
    .listen(tcp_listener)?
    .run();

    Ok(server)
}
