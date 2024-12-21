use super::{LinkReplacer, LinkReplacerConfig, ReplaceResult};
use fancy_regex::Regex;
use tracing::{debug, instrument};
use url::Url;

#[derive(Debug, Clone)]
pub struct YoutubeReplacer {
    new_domain: String,
    regex: Regex,
    domain_regex: Regex,
    strip_query: bool
}

const YOUTUBE_LINK_RE_STR: &'static str = r"https?://(www\.)?youtube\.com/shorts/[^\s]+";
const YOUTUBE_DOMAIN_RE_STR: &'static str = r"(www\.)?(youtube\.com/shorts/)";

impl YoutubeReplacer {
    pub fn new(config: &LinkReplacerConfig) -> ReplaceResult<Self> {
        let new_domain = config.new_domain.to_owned();
        let regex = Regex::new(config.regex.as_deref().unwrap_or(YOUTUBE_LINK_RE_STR))?;
        let domain_regex= Regex::new(config
            .domain_re
            .as_deref()
            .unwrap_or(YOUTUBE_DOMAIN_RE_STR))?;
        let strip_query = config.strip_query.unwrap_or(true);

        Ok(Self { new_domain, regex, domain_regex, strip_query })
    }
}

impl LinkReplacer for YoutubeReplacer {
    fn get_regex(&self) -> &Regex {
        &self.regex
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming Youtube Shorts URL...");
        let new_url = self
            .domain_regex
            .replace(url, format!("{}/", self.new_domain))
            .to_string();
        debug! {%new_url, "new url"};
        let mut url = Url::parse(&new_url)?;
        if self.strip_query {
            url.set_query(None);
        };
        
        Ok(url.to_string())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> ReplaceResult<YoutubeReplacer> {
        YoutubeReplacer::new(&LinkReplacerConfig {
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
