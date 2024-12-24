use std::net::TcpListener;

use once_cell::sync::Lazy;
use super::{get_subscriber, init_subscriber};
use crate::{start_server, AtomicBotStatus, BotStatus};
use actix_web::web::Data;

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("debug".into());
    init_subscriber(subscriber);
});

pub async fn init_tests() {
    Lazy::force(&TRACING);
}

pub struct TestServer {
    pub address: String,
    pub status: Data<AtomicBotStatus>
}

pub async fn spawn_test_server() -> TestServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!{"http://127.0.0.1:{}", port};
    let status = Data::new(AtomicBotStatus::new(BotStatus::Starting));

    let server = start_server(listener, status.clone()).expect("failed to bind address");
    let _ = tokio::spawn(server);
    TestServer {
        address,
        status
    }
}