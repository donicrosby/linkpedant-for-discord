use super::{LinkProcessor, LinkReplacer, LinkReplacerConfig, ReplaceResult};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct InstagramReplacer {
    inner: LinkProcessor,
}

const INSTAGRAM_LINK_RE_STR: &'static str =
    r"https?://(\w+\.)?instagram.com/(p|reel|stories)/[^\s]+";
const INSTAGRAM_DOMAIN_RE_STR: &'static str = r"(\w+\.)?(instagram\.com)";

impl InstagramReplacer {
    pub fn new(config: LinkReplacerConfig) -> ReplaceResult<Self> {
        let new_domain = config.new_domain;
        let regex_str = config.regex.unwrap_or(INSTAGRAM_LINK_RE_STR.to_string());
        let domain_re_str = config
            .domain_re
            .unwrap_or(INSTAGRAM_DOMAIN_RE_STR.to_string());
        let strip_query = config.strip_query.unwrap_or(true);
        let inner = LinkProcessor::new(new_domain, &regex_str, &domain_re_str, strip_query)?;
        Ok(Self { inner })
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> ReplaceResult<InstagramReplacer> {
        InstagramReplacer::new(LinkReplacerConfig {
            new_domain: "ddinstagram.com".into(),
            domain_re: None,
            regex: None,
            strip_query: None,
        })
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://www.instagram.com/reel/DCQBM9npSBK/?igsh=c2JxNzRidGk1bWhx";
        let expected = "https://ddinstagram.com/reel/DCQBM9npSBK/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
