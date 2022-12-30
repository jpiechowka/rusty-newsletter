use rusty_newsletter::startup::run_server;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to create TCP listener");
    run_server(tcp_listener)?.await
}
