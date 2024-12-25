use config::ConfigError;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

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

static DEFAULT_MAPPINGS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut default_mappings = HashMap::new();
    default_mappings.insert("instagram".into(), "ddinstagram.com".into());
    default_mappings.insert("pixiv".into(), "phixiv.net".into());
    default_mappings.insert("reddit".into(), "vxreddit.com".into());
    default_mappings.insert("tiktok".into(), "vxtiktok.com".into());
    default_mappings.insert("twitter".into(), "fxtwitter.com".into());
    default_mappings.insert("youtube".into(), "youtu.be".into());
    default_mappings.insert("bsky".into(), "bskyx.app".into());

    default_mappings
});

fn create_default_config(mut config: Config) -> Config {
    let replacer_config = &mut config.replacers;
    let defaults = Lazy::force(&DEFAULT_MAPPINGS);
    for (replacer_type, replace_domain) in defaults.iter() {
        replacer_config
            .entry(replacer_type.to_string())
            .or_insert_with(|| LinkReplacerConfig::new(replace_domain.to_string()));
    }
    config
}

pub type ReplacerConfig = HashMap<String, LinkReplacerConfig>;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub token: String,
    #[serde(default)]
    pub http: HttpConfig,
    pub reddit_media_regex: Option<String>,
    pub replacers: ReplacerConfig,
}

#[derive(Debug, Deserialize)]
pub struct LinkReplacerConfig {
    pub new_domain: String,
    pub regex: Option<String>,
    pub domain_re: Option<String>,
    pub strip_query: Option<bool>,
}

impl LinkReplacerConfig {
    pub fn new(new_domain: String) -> Self {
        Self {
            new_domain,
            regex: None,
            domain_re: None,
            strip_query: None,
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
