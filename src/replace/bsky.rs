use super::{
    LinkProcessor, LinkReplacer, LinkReplacerConfig, ProcessorConfig, ReplaceConfigError,
    ReplaceConfigResult, ReplaceResult,
};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct BskyReplacer {
    inner: LinkProcessor,
}

const BSKY_NEW_DOMAIN: &str = "bskyx.app";
const BSKY_LINK_RE_STR: &str =
    r"https?://bsky\.app/profile/((\w|\.|-)+|(did:plc:[234567a-z]{24}))/post/[234567a-z]{13}(?!/)";
const BSKY_DOMAIN_RE_STR: &str = r"bsky\.app";

pub fn bsky_default_new_domain() -> String {
    BSKY_NEW_DOMAIN.to_owned()
}

pub fn bsky_default_link_re_str() -> &'static str {
    BSKY_LINK_RE_STR
}

pub fn bsky_default_domain_re_str() -> &'static str {
    BSKY_DOMAIN_RE_STR
}

pub fn bsky_default_strip_query() -> bool {
    true
}

impl BskyReplacer {
    pub fn new(config: BskyConfig) -> Self {
        let inner = LinkProcessor::new(config.into());
        Self { inner }
    }
}

impl LinkReplacer for BskyReplacer {
    fn get_regex(&self) -> &Regex {
        self.inner.get_regex()
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming Instagram URL...");
        self.inner.transform_url(url)
    }
}

pub struct BskyConfig {
    inner: ProcessorConfig,
}

impl BskyConfig {
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

impl From<BskyConfig> for ProcessorConfig {
    fn from(value: BskyConfig) -> Self {
        value.inner
    }
}

impl TryFrom<&LinkReplacerConfig> for BskyConfig {
    type Error = ReplaceConfigError;
    fn try_from(value: &LinkReplacerConfig) -> Result<Self, Self::Error> {
        Self::new(
            value
                .new_domain
                .clone()
                .unwrap_or(bsky_default_new_domain()),
            value.regex.as_deref().unwrap_or(bsky_default_link_re_str()),
            value
                .domain_re
                .as_deref()
                .unwrap_or(bsky_default_domain_re_str()),
            value.strip_query.unwrap_or(bsky_default_strip_query()),
        )
    }
}

impl Default for BskyConfig {
    fn default() -> Self {
        Self::new(
            bsky_default_new_domain(),
            bsky_default_link_re_str(),
            bsky_default_domain_re_str(),
            bsky_default_strip_query(),
        )
        .unwrap()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> BskyReplacer {
        BskyReplacer::new(BskyConfig::default())
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://bsky.app/profile/albertflasher.bsky.social/post/3ldpen4om622h";
        let expected = "https://bskyx.app/profile/albertflasher.bsky.social/post/3ldpen4om622h";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
