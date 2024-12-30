use super::{
    LinkProcessor, LinkReplacer, LinkReplacerConfig, ProcessorConfig, ReplaceConfigError,
    ReplaceConfigResult, ReplaceResult,
};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct InstagramReplacer {
    inner: LinkProcessor,
}

const INSTAGRAM_NEW_DOMAIN: &str = "ddinstagram.com";
const INSTAGRAM_LINK_RE_STR: &str = r"https?://(\w+\.)?instagram.com/(p|reel|stories)/[^\s]+";
const INSTAGRAM_DOMAIN_RE_STR: &str = r"(\w+\.)?(instagram\.com)";

pub fn instagram_default_new_domain() -> String {
    INSTAGRAM_NEW_DOMAIN.to_owned()
}

pub fn instagram_default_link_re_str() -> &'static str {
    INSTAGRAM_LINK_RE_STR
}

pub fn instagram_default_domain_re_str() -> &'static str {
    INSTAGRAM_DOMAIN_RE_STR
}

pub fn instagram_default_strip_query() -> bool {
    true
}

impl InstagramReplacer {
    pub fn new(config: InstagramConfig) -> Self {
        let inner = LinkProcessor::new(config.into());
        Self { inner }
    }
}

impl LinkReplacer for InstagramReplacer {
    fn get_regex(&self) -> &Regex {
        self.inner.get_regex()
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming Instagram URL...");
        self.inner.transform_url(url)
    }
}

pub struct InstagramConfig {
    inner: ProcessorConfig,
}

impl InstagramConfig {
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

impl From<InstagramConfig> for ProcessorConfig {
    fn from(value: InstagramConfig) -> Self {
        value.inner
    }
}

impl TryFrom<&LinkReplacerConfig> for InstagramConfig {
    type Error = ReplaceConfigError;
    fn try_from(value: &LinkReplacerConfig) -> Result<Self, Self::Error> {
        Self::new(
            value
                .new_domain
                .clone()
                .unwrap_or(instagram_default_new_domain()),
            value
                .regex
                .as_deref()
                .unwrap_or(instagram_default_link_re_str()),
            value
                .domain_re
                .as_deref()
                .unwrap_or(instagram_default_domain_re_str()),
            value.strip_query.unwrap_or(instagram_default_strip_query()),
        )
    }
}

impl Default for InstagramConfig {
    fn default() -> Self {
        Self::new(
            instagram_default_new_domain(),
            instagram_default_link_re_str(),
            instagram_default_domain_re_str(),
            instagram_default_strip_query(),
        )
        .unwrap()
    }
}

impl AsRef<ProcessorConfig> for InstagramConfig {
    fn as_ref(&self) -> &ProcessorConfig {
        &self.inner
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> InstagramReplacer {
        InstagramReplacer::new(InstagramConfig::default())
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://www.instagram.com/reel/DCQBM9npSBK/?igsh=c2JxNzRidGk1bWhx";
        let expected = "https://ddinstagram.com/reel/DCQBM9npSBK/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
