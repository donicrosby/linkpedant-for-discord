use super::{LinkProcessor, LinkReplacer, LinkReplacerConfig, ReplaceResult};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct TwitterReplacer {
    inner: LinkProcessor,
}

const TWITTER_LINK_RE_STR: &'static str = r"https?://(x|twitter)\.com/(\w){1,15}/status/[^\s]+";
const TWITTER_DOMAIN_RE_STR: &'static str = r"(x|twitter)\.com";

impl TwitterReplacer {
    pub fn new(config: &LinkReplacerConfig) -> ReplaceResult<Self> {
        let new_domain = &config.new_domain;
        let regex_str = config.regex.as_deref().unwrap_or(TWITTER_LINK_RE_STR);
        let domain_re_str = config
            .domain_re
            .as_deref()
            .unwrap_or(TWITTER_DOMAIN_RE_STR);
        let strip_query = config.strip_query.unwrap_or(true);
        let inner = LinkProcessor::new(new_domain, regex_str, domain_re_str, strip_query)?;
        Ok(Self { inner })
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> ReplaceResult<TwitterReplacer> {
        TwitterReplacer::new(&LinkReplacerConfig {
            new_domain: "fxtwitter.com".into(),
            domain_re: None,
            regex: None,
            strip_query: None,
        })
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://x.com/PhillyD/status/1870093335936823564/";
        let expected = "https://fxtwitter.com/PhillyD/status/1870093335936823564/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
