use rusty_newsletter::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::run_server,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let tracing_subscriber =
        get_tracing_subscriber("rusty-newsletter".into(), "info".into(), std::io::stdout);
    init_tracing_subscriber(tracing_subscriber);

    let configuration = get_configuration().expect("Failed to read application configuration");
    let db_conn_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );

    let serve_address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let tcp_listener = TcpListener::bind(serve_address)?;
    run_server(tcp_listener, db_conn_pool, email_client)?.await?;

    Ok(())
}
