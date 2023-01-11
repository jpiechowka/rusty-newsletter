use rusty_newsletter::{config::get_config, startup::run_server};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    LogTracer::init().expect("Failed to initialize logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("rusty-newsletter".into(), std::io::stdout);
    let tracing_subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(tracing_subscriber).expect("Failed to set tracing subscriber");

    let config = get_config().expect("Failed to read application configuration");
    let db_conn_pool = PgPool::connect(&config.db_settings.connection_string())
        .await
        .expect("Failed to establish PostgreSQL connection");

    let serve_address = format!("127.0.0.1:{}", config.application_port);
    let tcp_listener = TcpListener::bind(serve_address).expect("Failed to create TCP listener");
    run_server(tcp_listener, db_conn_pool)?.await?;

    Ok(())
}
