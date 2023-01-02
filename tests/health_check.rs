use rusty_newsletter::{config::get_config, startup::run_server};
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use test_case::test_case;

pub struct TestApp {
    pub serve_address: String,
    pub db_conn_pool: PgPool,
}

fn spawn_app() -> TestApp {
    // Port zero will provide random port from the OS
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to create TCP listener");
    let port = tcp_listener.local_addr().unwrap().port();
    let serve_address = format!("http://127.0.0.1:{}", port);
    let mut config = get_configuration().expect("Failed to read configuration.");
    config.database.database_name = Uuid::new_v4().to_string();
    let db_conn_pool = configure_database(&config.database).await;

    let server =
        run_server(tcp_listener, db_conn_pool.clone()).expect("Failed to start server for testing");

    let _ = tokio::spawn(server);
    TestApp {
        serve_address,
        db_conn_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to establish PostgreSQL connection");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to establish PostgreSQL connection");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let http_client = reqwest::Client::new();

    let response = http_client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_status_for_valid_form_data() {
    let address = spawn_app();
    let config = get_config().expect("Failed to read application configuration");
    let db_conn_string = config.db_settings.connection_string();
    let db_conn = PgConnection::connect(&db_conn_string)
        .await
        .expect("Failed to establish PostgreSQL connection");

    let http_client = reqwest::Client::new();

    let body = "name=testy%20mctest&email=testy.mctest%40example.com";
    let response = http_client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(200, response.status().as_u16());

    let saved_data = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut db_conn)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved_data.email, "testy.mctest@example.com");
    assert_eq!(saved_data.name, "testy mctest");
}

#[tokio::test]
async fn subscribe_returns_404_status_for_invalid_http_method() {
    let address = spawn_app();
    let http_client = reqwest::Client::new();

    let body = "name=testy%20mctest&email=testy.mctest%40example.com";
    let response = http_client
        .get(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(404, response.status().as_u16());
}

#[test_case("name=testy%20mctest"; "missing email")]
#[test_case("email=testy.mctest%40example.com"; "missing name")]
#[test_case(""; "missing email and name")]
#[tokio::test]
async fn subscribe_returns_400_status_when_data_is_incorrect(invalid_body: &'static str) {
    let address = spawn_app();
    let http_client = reqwest::Client::new();

    let response = http_client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(400, response.status().as_u16());
}
