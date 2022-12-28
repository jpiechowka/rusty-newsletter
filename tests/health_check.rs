use std::net::TcpListener;

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

fn spawn_app() -> String {
    // Port zero will provide random port from the OS
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to create TCP listener");
    let port = tcp_listener.local_addr().unwrap().port();
    let server =
        rusty_newsletter::run_server(tcp_listener).expect("Failed to start server for testing");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
