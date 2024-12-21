use super::{LinkProcessor, LinkReplacer, LinkReplacerConfig, ReplaceResult};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct BskyReplacer {
    inner: LinkProcessor,
}

const BSKY_LINK_RE_STR: &str =
    r"https?://bsky\.app/profile/((\w|\.|-)+|(did:plc:[234567a-z]{24}))/post/[234567a-z]{13}(?!/)";
const BSKY_DOMAIN_RE_STR: &str = r"bsky\.app";

impl BskyReplacer {
    pub fn new(config: &LinkReplacerConfig) -> ReplaceResult<Self> {
        let new_domain = &config.new_domain;
        let regex_str = config.regex.as_deref().unwrap_or(BSKY_LINK_RE_STR);
        let domain_re_str = config.domain_re.as_deref().unwrap_or(BSKY_DOMAIN_RE_STR);
        let strip_query = config.strip_query.unwrap_or(true);
        let inner = LinkProcessor::new(new_domain, regex_str, domain_re_str, strip_query)?;
        Ok(Self { inner })
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> ReplaceResult<BskyReplacer> {
        BskyReplacer::new(&LinkReplacerConfig {
            new_domain: "bskyx.app".into(),
            domain_re: None,
            regex: None,
            strip_query: None,
        })
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://bsky.app/profile/albertflasher.bsky.social/post/3ldpen4om622h";
        let expected = "https://bskyx.app/profile/albertflasher.bsky.social/post/3ldpen4om622h";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
