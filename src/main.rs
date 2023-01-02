use rusty_newsletter::{config::get_config, startup::run_server};
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_config().expect("Failed to read application configuration");
    let db_conn_pool = PgPool::connect(&config.db_settings.connection_string())
        .await
        .expect("Failed to establish PostgreSQL connection");

    let serve_address = format!("127.0.0.1:{}", config.application_port);
    let tcp_listener = TcpListener::bind(serve_address).expect("Failed to create TCP listener");
    run_server(tcp_listener, db_conn_pool)?.await?;

    Ok(())
}
