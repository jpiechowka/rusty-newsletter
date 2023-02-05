use crate::helpers::spawn_app;
use test_case::test_case;

#[tokio::test]
async fn subscribe_returns_200_status_for_valid_form_data() {
    let app = spawn_app().await;
    let http_client = reqwest::Client::new();

    let body = "name=testy%20mctest&email=testy.mctest%40example.com";

    let response = http_client
        .post(&format!("{}/subscriptions", &app.serve_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(200, response.status().as_u16());

    let saved_data = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_conn_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved_data.email, "testy.mctest@example.com");
    assert_eq!(saved_data.name, "testy mctest");
}

#[tokio::test]
async fn subscribe_returns_404_status_for_invalid_http_method() {
    let app = spawn_app().await;
    let http_client = reqwest::Client::new();

    let body = "name=testy%20mctest&email=testy.mctest%40example.com";

    let response = http_client
        .get(&format!("{}/subscriptions", &app.serve_address))
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
#[test_case("name=&email=testy.mctest%40example.com"; "empty name")]
#[test_case("name=testy%20mctest&email="; "empty email")]
#[test_case("name=testy%20mctest&email=invalid-email"; "invalid email")]
#[tokio::test]
async fn subscribe_returns_400_status_when_data_is_incorrect(invalid_body: &'static str) {
    let app = spawn_app().await;
    let http_client = reqwest::Client::new();

    let response = http_client
        .post(&format!("{}/subscriptions", &app.serve_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(400, response.status().as_u16());
}
