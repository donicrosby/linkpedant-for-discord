use super::{
    LinkProcessor, LinkReplacer, LinkReplacerConfig, ProcessorConfig, ReplaceConfigError,
    ReplaceConfigResult, ReplaceResult,
};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct TikTokReplacer {
    inner: LinkProcessor,
}

const TIKTOK_NEW_DOMAIN: &str = "d.tnktok.com";
const TIKTOK_LINK_RE_STR: &str = r"https?://(\w+\.)?tiktok\.com/((t/)?\w+|@[^\s]+/video)";
const TIKTOK_DOMAIN_RE_STR: &str = r"([w]{3}\.)?tiktok\.com";

pub fn tiktok_default_new_domain() -> String {
    TIKTOK_NEW_DOMAIN.to_owned()
}

pub fn tiktok_default_link_re_str() -> &'static str {
    TIKTOK_LINK_RE_STR
}

pub fn tiktok_default_domain_re_str() -> &'static str {
    TIKTOK_DOMAIN_RE_STR
}

pub fn tiktok_default_strip_query() -> bool {
    true
}

impl TikTokReplacer {
    pub fn new(config: TikTokConfig) -> Self {
        let inner = LinkProcessor::new(config.into());
        Self { inner }
    }
}

impl LinkReplacer for TikTokReplacer {
    fn get_regex(&self) -> &Regex {
        self.inner.get_regex()
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming TikTok URL...");
        self.inner.transform_url(url)
    }
}

pub struct TikTokConfig {
    inner: ProcessorConfig,
}

impl TikTokConfig {
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

impl From<TikTokConfig> for ProcessorConfig {
    fn from(value: TikTokConfig) -> Self {
        value.inner
    }
}

impl TryFrom<&LinkReplacerConfig> for TikTokConfig {
    type Error = ReplaceConfigError;
    fn try_from(value: &LinkReplacerConfig) -> Result<Self, Self::Error> {
        Self::new(
            value
                .new_domain
                .clone()
                .unwrap_or(tiktok_default_new_domain()),
            value
                .regex
                .as_deref()
                .unwrap_or(tiktok_default_link_re_str()),
            value
                .domain_re
                .as_deref()
                .unwrap_or(tiktok_default_domain_re_str()),
            value.strip_query.unwrap_or(tiktok_default_strip_query()),
        )
    }
}

impl Default for TikTokConfig {
    fn default() -> Self {
        Self::new(
            tiktok_default_new_domain(),
            tiktok_default_link_re_str(),
            tiktok_default_domain_re_str(),
            tiktok_default_strip_query(),
        )
        .unwrap()
    }
}

impl AsRef<ProcessorConfig> for TikTokConfig {
    fn as_ref(&self) -> &ProcessorConfig {
        &self.inner
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> TikTokReplacer {
        TikTokReplacer::new(TikTokConfig::default())
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://www.tiktok.com/t/ZTYXjHYeg/";
        let expected = "https://d.tnktok.com/t/ZTYXjHYeg/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
