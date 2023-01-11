use rusty_newsletter::{
    config::get_config,
    startup::run_server,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let tracing_subscriber =
        get_tracing_subscriber("rusty-newsletter".into(), "info".into(), std::io::stdout);
    init_tracing_subscriber(tracing_subscriber);

    let config = get_config().expect("Failed to read application configuration");
    let db_conn_pool = PgPool::connect(&config.db_settings.connection_string())
        .await
        .expect("Failed to establish PostgreSQL connection");

    let serve_address = format!("127.0.0.1:{}", config.application_port);
    let tcp_listener = TcpListener::bind(serve_address).expect("Failed to create TCP listener");
    run_server(tcp_listener, db_conn_pool)?.await?;

    Ok(())
}
