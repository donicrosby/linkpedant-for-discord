use super::{
    LinkProcessor, LinkReplacer, LinkReplacerConfig, ProcessorConfig, ReplaceConfigError,
    ReplaceConfigResult, ReplaceResult,
};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct PixivReplacer {
    inner: LinkProcessor,
}

const PIXIV_NEW_DOMAIN: &str = "phixiv.net";
const PIXIV_LINK_RE_STR: &str = r"https?://(\w+\.)?pixiv\.net/(\w+/)?(artworks|member_illust\.php)(/|\?illust_id=)\d+(/?\d+)?[^\s]+";
const PIXIV_DOMAIN_RE_STR: &str = r"(\w+\.)?(pixiv\.net)";

pub fn pixiv_default_new_domain() -> String {
    PIXIV_NEW_DOMAIN.to_owned()
}

pub fn pixiv_default_link_re_str() -> &'static str {
    PIXIV_LINK_RE_STR
}

pub fn pixiv_default_domain_re_str() -> &'static str {
    PIXIV_DOMAIN_RE_STR
}

pub fn pixiv_default_strip_query() -> bool {
    false
}

impl PixivReplacer {
    pub fn new(config: PixivConfig) -> Self {
        let inner = LinkProcessor::new(config.into());
        Self { inner }
    }
}

impl LinkReplacer for PixivReplacer {
    fn get_regex(&self) -> &Regex {
        self.inner.get_regex()
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming Pixiv URL...");
        self.inner.transform_url(url)
    }
}

pub struct PixivConfig {
    inner: ProcessorConfig,
}

impl PixivConfig {
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

impl From<PixivConfig> for ProcessorConfig {
    fn from(value: PixivConfig) -> Self {
        value.inner
    }
}

impl TryFrom<&LinkReplacerConfig> for PixivConfig {
    type Error = ReplaceConfigError;
    fn try_from(value: &LinkReplacerConfig) -> Result<Self, Self::Error> {
        Self::new(
            value
                .new_domain
                .clone()
                .unwrap_or(pixiv_default_new_domain()),
            value
                .regex
                .as_deref()
                .unwrap_or(pixiv_default_link_re_str()),
            value
                .domain_re
                .as_deref()
                .unwrap_or(pixiv_default_domain_re_str()),
            value.strip_query.unwrap_or(pixiv_default_strip_query()),
        )
    }
}

impl Default for PixivConfig {
    fn default() -> Self {
        Self::new(
            pixiv_default_new_domain(),
            pixiv_default_link_re_str(),
            pixiv_default_domain_re_str(),
            pixiv_default_strip_query(),
        )
        .unwrap()
    }
}
#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> PixivReplacer {
        PixivReplacer::new(PixivConfig::default())
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://www.pixiv.net/en/artworks/125183260";
        let expected = "https://phixiv.net/en/artworks/125183260";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
