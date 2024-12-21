use super::{LinkProcessor, LinkReplacer, LinkReplacerConfig, ReplaceResult};
use fancy_regex::Regex;
use tracing::{debug, instrument};
use url::Url;

#[derive(Debug, Clone)]
pub struct YoutubeReplacer {
    inner: LinkProcessor,
}

const YOUTUBE_LINK_RE_STR: &'static str = r"https?://(www\.)?youtube\.com/shorts/[^\s]+";
const YOUTUBE_DOMAIN_RE_STR: &'static str = r"(www\.)?(youtube\.com/shorts/)";

impl YoutubeReplacer {
    pub fn new(config: LinkReplacerConfig) -> ReplaceResult<Self> {
        let new_domain = config.new_domain;
        let regex_str = config.regex.unwrap_or(YOUTUBE_LINK_RE_STR.to_string());
        let domain_re_str = config
            .domain_re
            .unwrap_or(YOUTUBE_DOMAIN_RE_STR.to_string());
        let strip_query = config.strip_query.unwrap_or(true);
        let inner = LinkProcessor::new(new_domain, &regex_str, &domain_re_str, strip_query)?;
        Ok(Self { inner })
    }
}

impl LinkReplacer for YoutubeReplacer {
    fn get_regex(&self) -> &Regex {
        self.inner.get_regex()
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming Youtube Shorts URL...");
        let new_url = self
            .inner
            .domain_regex()
            .replace(url, format!("{}/", self.inner.new_domain()))
            .to_string();
        debug! {%new_url, "new url"};
        Ok(new_url)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> ReplaceResult<YoutubeReplacer> {
        YoutubeReplacer::new(LinkReplacerConfig {
            new_domain: "youtu.be".into(),
            domain_re: None,
            regex: None,
            strip_query: None,
        })
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://youtube.com/shorts/xFnfOdb35FI/";
        let expected = "https://youtu.be/xFnfOdb35FI/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
