use rust_i18n;
use serenity::{all::GatewayIntents, Client};
use thiserror::Error;

pub(crate) use client::Handler;
pub(crate) use commands::LinkPedantCommands;
pub use config::{get_configuration, Config};
use tracing::{error, warn};

mod client;
mod commands;
mod config;
mod util;

rust_i18n::i18n!("locales");

pub type Result<T> = ::core::result::Result<T, LinkPedantError>;

#[derive(Debug, Error)]
pub enum LinkPedantError {
    #[error("serenity error")]
    Serenity(#[from] serenity::Error),
    #[error("config error")]
    Config(#[from] ::config::ConfigError),
}

pub struct LinkPedant {
    config: Config,
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
        Ok(Self { config, client })
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
