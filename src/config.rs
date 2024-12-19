use config::ConfigError;
use serde::Deserialize;

pub fn get_configuration() -> Result<Config, ConfigError> {
    let config = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;
    config.try_deserialize::<Config>()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub token: String,
}
