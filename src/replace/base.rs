use core::fmt::Debug;
use fancy_regex::{Captures, Regex};
use thiserror::Error;
use tracing::{debug, info, instrument, warn};
use url::Url;

#[derive(Debug, Error, Clone)]
pub enum ReplaceError {
    #[error("url error")]
    Url(#[from] url::ParseError),

    #[error("url contains no host")]
    UrlHost,

    #[error("url was not modified")]
    UrlNotModified(String),

    #[error("no query params available")]
    NoQueryParams,

    #[error("utf8 decode")]
    Utf8Decode,

    #[error("regex missing named capture group")]
    MissingGroup(String),

    #[error("config error")]
    Config(#[from] ReplaceConfigError),
}

#[derive(Debug, Error, Clone)]
pub enum ReplaceConfigError {
    #[error("regex error")]
    Regex(#[from] fancy_regex::Error),

    #[error("missing config option")]
    MissingOption(String),

    #[error("invalid custom replacer")]
    InvalidReplacer(String),
}

pub type ReplaceResult<T> = std::result::Result<T, ReplaceError>;
pub type ReplaceConfigResult<T> = std::result::Result<T, ReplaceConfigError>;

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub new_domain: String,
    pub link_regex: Regex,
    pub domain_regex: Regex,
    pub strip_query: bool,
}

impl ProcessorConfig {
    pub fn new(
        new_domain: String,
        link_regex: &str,
        domain_regex: &str,
        strip_query: bool,
    ) -> ReplaceConfigResult<Self> {
        let link_regex = Regex::new(link_regex)?;
        let domain_regex = Regex::new(domain_regex)?;
        Ok(Self {
            new_domain,
            link_regex,
            domain_regex,
            strip_query,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LinkProcessor {
    config: ProcessorConfig,
}

impl LinkProcessor {
    pub fn new(config: ProcessorConfig) -> Self {
        Self { config }
    }
}

impl LinkReplacer for LinkProcessor {
    fn get_regex(&self) -> &Regex {
        &self.config.link_regex
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Parsing URL...");
        let mut url = Url::parse(url)?;
        let new_host = url.host_str().ok_or(ReplaceError::UrlHost).map(|h| {
            self.config
                .domain_regex
                .replace(h, &self.config.new_domain)
                .to_string()
        })?;
        debug! {%new_host, "setting new host"};
        url.set_host(Some(&new_host))?;
        if self.config.strip_query {
            url.set_query(None);
        }
        let new_url = url.to_string();
        debug! {%new_url, "new url"};
        Ok(new_url)
    }
}

pub trait LinkReplacer: Debug {
    fn get_regex(&self) -> &Regex;

    fn is_match(&self, url: &str) -> ReplaceResult<bool> {
        self.get_regex()
            .is_match(url)
            .map_err(|err| ReplaceError::Config(err.into()))
    }

    #[instrument(skip(self))]
    fn process_url(&self, url: &str) -> ReplaceResult<String> {
        let re = self.get_regex();
        let mut process_error = None;
        debug!("Processing URL...");
        let new_url = re
            .replace(url, |caps: &Captures<'_>| {
                let orig_url = &caps[0];
                self.transform_url(orig_url)
                    .and_then(|processed_url| {
                        if processed_url.eq(orig_url) {
                            let err = ReplaceError::UrlNotModified(processed_url);
                            process_error = Some(err.clone());
                            Err(err)
                        } else {
                            info! {%orig_url, %processed_url, "replaced url"};
                            Ok(processed_url)
                        }
                    })
                    .map_err(|err| warn! {%err, "could not transform url"})
                    .unwrap_or(orig_url.to_string())
            })
            .to_string();
        if let Some(err) = process_error {
            Err(err)
        } else {
            Ok(new_url)
        }
    }

    fn transform_url(&self, url: &str) -> ReplaceResult<String>;
}
