use once_cell::sync::Lazy;
use rusty_newsletter::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    startup::run_server,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            get_tracing_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_tracing_subscriber(subscriber);
    } else {
        let subscriber =
            get_tracing_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_tracing_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub serve_address: String,
    pub db_conn_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // Port zero will provide random port from the OS
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to create TCP listener");
    let port = tcp_listener.local_addr().unwrap().port();
    let serve_address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read application configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_conn_pool = configure_database(&configuration.database).await;

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let server = run_server(tcp_listener, db_conn_pool.clone(), email_client)
        .expect("Failed to start server for testing");

    let _ = tokio::spawn(server);
    TestApp {
        serve_address,
        db_conn_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to establish PostgreSQL connection");

    // Create database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to establish PostgreSQL connection");

    // Migrate database
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
