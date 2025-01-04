use config::ConfigError;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

use crate::AmazonConfig;

pub fn get_configuration() -> Result<Config, ConfigError> {
    let config = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(
            config::Environment::with_prefix("BOT")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    config
        .try_deserialize::<Config>()
        .map(create_default_config)
}

static DEFAULT_MAPPINGS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    let default_mappings = vec![
        "instagram",
        "pixiv",
        "reddit",
        "tiktok",
        "twitter",
        "youtube",
        "bsky",
    ];

    default_mappings
});

fn create_default_config(mut config: Config) -> Config {
    let replacer_config = &mut config.replacers;
    let defaults = Lazy::force(&DEFAULT_MAPPINGS);
    for replacer_type in defaults.iter() {
        replacer_config
            .entry(replacer_type.to_string())
            .or_default();
    }
    config
}

pub type ReplacerConfig = HashMap<String, LinkReplacerConfig>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteReplyReaction(String);

impl DeleteReplyReaction {
    pub fn new(str: String) -> Self {
        Self(str)
    }
}

impl AsRef<str> for DeleteReplyReaction {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for DeleteReplyReaction {
    fn default() -> Self {
        Self(String::from("❌"))
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub token: String,
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub amazon: AmazonConfig,
    pub reddit_media_regex: Option<String>,
    #[serde(default)]
    pub delete_reply_reaction: DeleteReplyReaction,
    pub replacers: ReplacerConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct LinkReplacerConfig {
    pub new_domain: Option<String>,
    pub regex: Option<String>,
    pub domain_re: Option<String>,
    pub strip_query: Option<bool>,
    #[serde(flatten)]
    pub custom_config: HashMap<String, String>,
}

impl LinkReplacerConfig {
    pub fn new(new_domain: String) -> Self {
        Self {
            new_domain: Some(new_domain),
            regex: None,
            domain_re: None,
            strip_query: None,
            custom_config: HashMap::new(),
        }
    }

    pub fn set_regex(&mut self, regex: String) -> &mut Self {
        self.regex = Some(regex);
        self
    }

    pub fn set_domain_re(&mut self, domain_re: String) -> &mut Self {
        self.domain_re = Some(domain_re);
        self
    }

    pub fn set_strip_query(&mut self, strip_query: bool) -> &mut Self {
        self.strip_query = Some(strip_query);
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
}

impl Default for HttpConfig {
    fn default() -> Self {
        let host = "127.0.0.1".into();
        let port = 3000;
        Self { host, port }
    }
}
