use super::{LinkProcessor, LinkReplacer, LinkReplacerConfig, ReplaceResult};
use fancy_regex::Regex;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct RedditReplacer {
    inner: LinkProcessor,
}

const REDDIT_LINK_RE_STR: &'static str =
    r"https?://(redd.it|((\w+\.)?reddit.com/(r|u|user)/\w+/(s|comments))/)[^\s]+";
const REDDIT_DOMAIN_RE_STR: &'static str = r"(((\w+\.)?reddit\.com)|(redd\.it))";

impl RedditReplacer {
    pub fn new(config: LinkReplacerConfig) -> ReplaceResult<Self> {
        let new_domain = config.new_domain;
        let regex_str = config.regex.unwrap_or(REDDIT_LINK_RE_STR.to_string());
        let domain_re_str = config.domain_re.unwrap_or(REDDIT_DOMAIN_RE_STR.to_string());
        let strip_query = config.strip_query.unwrap_or(true);
        let inner = LinkProcessor::new(new_domain, &regex_str, &domain_re_str, strip_query)?;
        Ok(Self { inner })
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> ReplaceResult<RedditReplacer> {
        RedditReplacer::new(LinkReplacerConfig {
            new_domain: "vxreddit.com".into(),
            domain_re: None,
            regex: None,
            strip_query: None,
        })
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
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
