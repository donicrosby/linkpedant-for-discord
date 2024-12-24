use actix_web::web::Data;
use serenity::prelude::*;
use serenity::{all::GatewayIntents, Client};
use std::net::TcpListener;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, warn};

pub(crate) use client::Handler;
pub(crate) use commands::LinkPedantCommands;
pub use config::{get_configuration, Config, HttpConfig, LinkReplacerConfig, ReplacerConfig};
pub(crate) use replace::MessageProcessor;

mod client;
mod commands;
mod config;
mod http;
mod replace;
mod util;

rust_i18n::i18n!("locales");

#[cfg(test)]
pub(crate) use util::{init_tests, spawn_test_server};

pub use util::{get_subscriber, init_subscriber};

pub type Result<T> = ::core::result::Result<T, LinkPedantError>;

pub use http::{start_server, AtomicBotStatus, BotStatus, HttpError};

pub(crate) struct MessageHandler;

impl TypeMapKey for MessageHandler {
    type Value = Arc<RwLock<MessageProcessor>>;
}

pub(crate) struct BotState;

impl TypeMapKey for BotState {
    type Value = Data<AtomicBotStatus>;
}

#[derive(Debug, Error)]
pub enum LinkPedantError {
    #[error("serenity error")]
    Serenity(#[from] serenity::Error),
    #[error("config error")]
    Config(#[from] ::config::ConfigError),
    #[error("http error")]
    Http(#[from] HttpError),
}

pub struct LinkPedant {
    client: Client,
    http_config: HttpConfig,
    state: Data<AtomicBotStatus>,
}

impl LinkPedant {
    pub async fn new(config: Config) -> Result<Self> {
        let intents = GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::GUILD_MESSAGES;
        let state = Data::new(AtomicBotStatus::new(BotStatus::Starting));
        let http_config = config.http;
        let client = Client::builder(&config.token, intents)
            .event_handler(Handler)
            .await
            .map_err(|err| {
                error! {%err, "could not create client"}
                err
            })?;
        {
            let mut data = client.data.write().await;
            data.insert::<MessageHandler>(Arc::new(RwLock::new(MessageProcessor::new(
                &config.replacers,
                config.reddit_media_regex.to_owned(),
            ))));
            data.insert::<BotState>(state.clone());
        }
        Ok(Self {
            client,
            http_config,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let listener = TcpListener::bind((self.http_config.host.clone(), self.http_config.port))
            .expect("could not bind to port");
        let server = start_server(listener, self.state.clone())?;
        tokio::select! {
            server_res = server => {
                if let Err(why) = server_res {
                    warn! {
                        %why,
                        "http server error"
                    };
                }
            }
            bot_res = self.client.start_autosharded() => {
                if let Err(why) = bot_res {
                    warn! {
                        %why,
                        "client error"
                    };
                }
            }
        }

        self.client.shard_manager.shutdown_all().await;
        Ok(())
    }
}
