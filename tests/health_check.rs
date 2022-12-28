use std::net::TcpListener;
use test_case::test_case;

fn spawn_app() -> String {
    // Port zero will provide random port from the OS
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to create TCP listener");
    let port = tcp_listener.local_addr().unwrap().port();
    let server =
        rusty_newsletter::run_server(tcp_listener).expect("Failed to start server for testing");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
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
    let http_client = reqwest::Client::new();

    let body = "name=testy%20mctest&email=testy.mctest%40example.com";
    let response = http_client
        .get(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(200, response.status().as_u16());
}

#[test_case("name=testy%20mctest"; "missing email")]
#[test_case("email=testy.mctest%40example.com"; "missing name")]
#[test_case(""; "missing email and name")]
#[tokio::test]
async fn subscribe_returns_400_status_when_data_is_incorrect(invalid_body: &'static str) {
    let address = spawn_app();
    let http_client = reqwest::Client::new();

    let response = http_client
        .get(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(400, response.status().as_u16());
}
