use rusty_newsletter::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let subscriber =
        get_tracing_subscriber("rusty-newsletter".into(), "info".into(), std::io::stdout);
    init_tracing_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read application configuration");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
