use serenity::prelude::*;
use serenity::{all::GatewayIntents, Client};
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, warn};

pub(crate) use client::Handler;
pub(crate) use commands::LinkPedantCommands;
pub use config::{get_configuration, Config, LinkReplacerConfig, ReplacerConfig};
pub(crate) use replace::MessageProcessor;

mod client;
mod commands;
mod config;
mod replace;
mod util;
mod http;

rust_i18n::i18n!("locales");

#[cfg(test)]
pub(crate) use util::{init_tests, spawn_test_server};

pub use util::{get_subscriber, init_subscriber};

pub type Result<T> = ::core::result::Result<T, LinkPedantError>;

pub use http::{start_server, HttpError, BotStatus, AtomicBotStatus};

pub(crate) struct MessageHandler;

impl TypeMapKey for MessageHandler {
    type Value = Arc<RwLock<MessageProcessor>>;
}

#[derive(Debug, Error)]
pub enum LinkPedantError {
    #[error("serenity error")]
    Serenity(#[from] serenity::Error),
    #[error("config error")]
    Config(#[from] ::config::ConfigError),
    #[error("http error")]
    Http(#[from] HttpError)
}

pub struct LinkPedant {
    client: Client,
}

impl LinkPedant {
    pub async fn new(config: Config) -> Result<Self> {
        let intents = GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::GUILD_MESSAGES;
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
                &config.replacer_config,
                config.reddit_media_regex.to_owned(),
            ))));
        }
        Ok(Self { client })
    }

    pub async fn run(&mut self) -> Result<()> {
        if let Err(why) = self.client.start_autosharded().await {
            warn! {
                %why,
                "client error"
            }
        }
        Ok(())
    }
}
