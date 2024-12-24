use actix_web::{body::BoxBody, web, HttpResponse, Responder};
use serenity::all::StatusCode;
use std::sync::atomic::Ordering;
use super::{AtomicBotStatus, BotStatus};


pub async fn health(status: web::Data<AtomicBotStatus>) -> impl Responder {
    match status.get_ref().load(Ordering::Relaxed) {
        BotStatus::Starting => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR).set_body(BoxBody::new("Starting")),
        BotStatus::Ready => HttpResponse::Ok().body("Healthy")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::spawn_test_server;

    #[tokio::test]
    async fn test_health_check() {
        let app = spawn_test_server().await;
        let client = reqwest::Client::new();

        let response = client.get(&format!("{}/health", &app.address)).send().await.expect("failed to execute request");

        assert!(response.status().is_server_error());
        assert_eq!("Starting", response.text().await.unwrap());
        
        {
            let status = app.status.get_ref();
            status.store(BotStatus::Ready, Ordering::Relaxed);
        }

        let response = client.get(&format!("{}/health", &app.address)).send().await.expect("failed to execute request");

        assert!(response.status().is_success());
        assert_eq!("Healthy", response.text().await.unwrap())
    }
}