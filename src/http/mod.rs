use atomic_enum::atomic_enum;
use thiserror::Error;
use std::net::TcpListener;
use std::io;
use actix_web::{dev::Server, web::{self, Data}, App, HttpServer};

mod health;
use health::health;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("actix io error")]
    Io(#[from] io::Error)
}


pub type HttpResult<T> = ::core::result::Result<T, HttpError>;

#[atomic_enum]
#[derive(PartialEq)]
pub enum BotStatus {
    Starting = 0,
    Ready
}

pub fn start_server(listener: TcpListener, bot_status: Data<AtomicBotStatus>) -> HttpResult<Server> {
    let server = HttpServer::new( move || {
        App::new()
            .route("/health", web::get().to(health))
            .app_data(Data::clone(&bot_status))
    }).listen(listener)?.run();
    Ok(server)
}