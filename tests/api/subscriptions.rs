use crate::helpers::spawn_app;
use test_case::test_case;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let body = "name=testy%20mctest&email=testy.mctest%40example.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let app = spawn_app().await;
    let body = "name=testy%20mctest&email=testy.mctest%40example.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_conn_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "testy.mctest@example.com");
    assert_eq!(saved.name, "testy mctest");
    assert_eq!(saved.status, "pending_confirmation");
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
    let response = app.post_subscriptions(invalid_body.into()).await;
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=testy%20mctest&email=testy.mctest%40example.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=testy%20mctest&email=testy.mctest%40example.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}
