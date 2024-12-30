use super::{
    LinkProcessor, LinkReplacer, LinkReplacerConfig, ProcessorConfig, ReplaceConfigError,
    ReplaceConfigResult, ReplaceResult,
};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct RedditReplacer {
    inner: LinkProcessor,
}

const REDDIT_NEW_DOMAIN: &str = "vxreddit.com";
const REDDIT_LINK_RE_STR: &str =
    r"https?://(redd.it|((\w+\.)?reddit.com/(r|u|user)/\w+/(s|comments))/)[^\s]+";
const REDDIT_DOMAIN_RE_STR: &str = r"(((\w+\.)?reddit\.com)|(redd\.it))";

pub fn reddit_default_new_domain() -> String {
    REDDIT_NEW_DOMAIN.to_owned()
}

pub fn reddit_default_link_re_str() -> &'static str {
    REDDIT_LINK_RE_STR
}

pub fn reddit_default_domain_re_str() -> &'static str {
    REDDIT_DOMAIN_RE_STR
}

pub fn reddit_default_strip_query() -> bool {
    true
}

impl RedditReplacer {
    pub fn new(config: RedditConfig) -> Self {
        let inner = LinkProcessor::new(config.into());
        Self { inner }
    }
}

impl LinkReplacer for RedditReplacer {
    fn get_regex(&self) -> &Regex {
        self.inner.get_regex()
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming Reddit URL...");
        self.inner.transform_url(url)
    }
}

pub struct RedditConfig {
    inner: ProcessorConfig,
}

impl RedditConfig {
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

impl From<RedditConfig> for ProcessorConfig {
    fn from(value: RedditConfig) -> Self {
        value.inner
    }
}

impl TryFrom<&LinkReplacerConfig> for RedditConfig {
    type Error = ReplaceConfigError;
    fn try_from(value: &LinkReplacerConfig) -> Result<Self, Self::Error> {
        Self::new(
            value
                .new_domain
                .clone()
                .unwrap_or(reddit_default_new_domain()),
            value
                .regex
                .as_deref()
                .unwrap_or(reddit_default_link_re_str()),
            value
                .domain_re
                .as_deref()
                .unwrap_or(reddit_default_domain_re_str()),
            value.strip_query.unwrap_or(reddit_default_strip_query()),
        )
    }
}

impl Default for RedditConfig {
    fn default() -> Self {
        Self::new(
            reddit_default_new_domain(),
            reddit_default_link_re_str(),
            reddit_default_domain_re_str(),
            reddit_default_strip_query(),
        )
        .unwrap()
    }
}

impl AsRef<ProcessorConfig> for RedditConfig {
    fn as_ref(&self) -> &ProcessorConfig {
        &self.inner
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> RedditReplacer {
        RedditReplacer::new(RedditConfig::default())
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://www.reddit.com/r/Unexpected/comments/1hivblz/pro_tip_for_girls_to_get_home_safely_at_night/";
        let expected = "https://vxreddit.com/r/Unexpected/comments/1hivblz/pro_tip_for_girls_to_get_home_safely_at_night/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);

        let url = "https://redd.it/6kq5hk";
        let expected = "https://vxreddit.com/6kq5hk";
        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
