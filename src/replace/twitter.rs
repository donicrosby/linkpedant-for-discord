use super::{
    LinkProcessor, LinkReplacer, LinkReplacerConfig, ProcessorConfig, ReplaceConfigError,
    ReplaceConfigResult, ReplaceResult,
};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct TwitterReplacer {
    inner: LinkProcessor,
}

const TWITTER_NEW_DOMAIN: &str = "fxtwitter.com";
const TWITTER_LINK_RE_STR: &str = r"https?://(x|twitter)\.com/(\w){1,15}/status/[^\s]+";
const TWITTER_DOMAIN_RE_STR: &str = r"(x|twitter)\.com";

pub fn twitter_default_new_domain() -> String {
    TWITTER_NEW_DOMAIN.to_owned()
}

pub fn twitter_default_link_re_str() -> &'static str {
    TWITTER_LINK_RE_STR
}

pub fn twitter_default_domain_re_str() -> &'static str {
    TWITTER_DOMAIN_RE_STR
}

pub fn twitter_default_strip_query() -> bool {
    true
}

impl TwitterReplacer {
    pub fn new(config: TwitterConfig) -> Self {
        let inner = LinkProcessor::new(config.into());
        Self { inner }
    }
}

impl LinkReplacer for TwitterReplacer {
    fn get_regex(&self) -> &Regex {
        self.inner.get_regex()
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming Twitter URL...");
        self.inner.transform_url(url)
    }
}

pub struct TwitterConfig {
    inner: ProcessorConfig,
}

impl TwitterConfig {
    pub fn new(
        new_domain: String,
        regex: &str,
        domain_regex: &str,
        strip_query: bool,
    ) -> ReplaceConfigResult<Self> {
        let config = ProcessorConfig::new(new_domain, regex, domain_regex, strip_query)?;
        Ok(Self { inner: config })
    }
}

impl From<TwitterConfig> for ProcessorConfig {
    fn from(value: TwitterConfig) -> Self {
        value.inner
    }
}

impl TryFrom<&LinkReplacerConfig> for TwitterConfig {
    type Error = ReplaceConfigError;
    fn try_from(value: &LinkReplacerConfig) -> Result<Self, Self::Error> {
        Self::new(
            value
                .new_domain
                .clone()
                .unwrap_or(twitter_default_new_domain()),
            value
                .regex
                .as_deref()
                .unwrap_or(twitter_default_link_re_str()),
            value
                .domain_re
                .as_deref()
                .unwrap_or(twitter_default_domain_re_str()),
            value.strip_query.unwrap_or(twitter_default_strip_query()),
        )
    }
}

impl Default for TwitterConfig {
    fn default() -> Self {
        Self::new(
            twitter_default_new_domain(),
            twitter_default_link_re_str(),
            twitter_default_domain_re_str(),
            twitter_default_strip_query(),
        )
        .unwrap()
    }
}

impl AsRef<ProcessorConfig> for TwitterConfig {
    fn as_ref(&self) -> &ProcessorConfig {
        &self.inner
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> TwitterReplacer {
        TwitterReplacer::new(TwitterConfig::default())
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://x.com/PhillyD/status/1870093335936823564/";
        let expected = "https://fxtwitter.com/PhillyD/status/1870093335936823564/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
