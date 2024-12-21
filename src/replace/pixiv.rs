use super::{LinkProcessor, LinkReplacer, LinkReplacerConfig, ReplaceResult};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct PixivReplacer {
    inner: LinkProcessor,
}

const PIXIV_LINK_RE_STR: &'static str = r"https?://(\w+\.)?pixiv\.net/(\w+/)?(artworks|member_illust\.php)(/|\?illust_id=)\d+(/?\d+)?[^\s]+";
const PIXIV_DOMAIN_RE_STR: &'static str = r"(\w+\.)?(pixiv\.net)";

impl PixivReplacer {
    pub fn new(config: &LinkReplacerConfig) -> ReplaceResult<Self> {
        let new_domain = &config.new_domain;
        let regex_str = config.regex.as_deref().unwrap_or(PIXIV_LINK_RE_STR);
        let domain_re_str = config.domain_re.as_deref().unwrap_or(PIXIV_DOMAIN_RE_STR);
        let strip_query = config.strip_query.unwrap_or(false);
        let inner = LinkProcessor::new(new_domain, regex_str, domain_re_str, strip_query)?;
        Ok(Self { inner })
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> ReplaceResult<PixivReplacer> {
        PixivReplacer::new(&LinkReplacerConfig {
            new_domain: "phixiv.net".into(),
            domain_re: None,
            regex: None,
            strip_query: None,
        })
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://www.pixiv.net/en/artworks/125183260";
        let expected = "https://phixiv.net/en/artworks/125183260";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
