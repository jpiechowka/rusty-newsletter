#[tokio::test]
async fn health_check_works() {
    spawn_app();

    let http_client = reqwest::Client::new();

    let response = http_client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() {
    let server = rusty_newsletter::run_server().expect("Failed to start server for testing");
    let _ = tokio::spawn(server);
}
