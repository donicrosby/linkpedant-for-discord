use super::{
    LinkReplacer, LinkReplacerConfig, ReplaceConfigError, ReplaceConfigResult, ReplaceError,
    ReplaceResult,
};
use fancy_regex::Regex;
use tracing::{debug, instrument};
use url::Url;

#[derive(Debug, Clone)]
pub struct YoutubeReplacer {
    config: YoutubeConfig,
}

const YOUTUBE_NEW_DOMAIN: &str = "youtu.be";
const YOUTUBE_LINK_RE_STR: &str =
    r"https?://((www|m)\.)?youtube\.com/(shorts/[^\s]+|watch\?(?:&?(?:[^\s]+\=[^\s]+))+)";
const YOUTUBE_DOMAIN_RE_STR: &str = r"((www|m)\.)?(youtube\.com/(shorts/|watch\?v=))";

pub fn youtube_default_new_domain() -> String {
    YOUTUBE_NEW_DOMAIN.to_owned()
}

pub fn youtube_default_link_re_str() -> String {
    YOUTUBE_LINK_RE_STR.to_owned()
}

pub fn youtube_default_domain_re_str() -> String {
    YOUTUBE_DOMAIN_RE_STR.to_owned()
}

pub fn youtube_default_strip_query() -> bool {
    true
}

impl YoutubeReplacer {
    pub fn new(config: YoutubeConfig) -> Self {
        Self { config }
    }
}

impl LinkReplacer for YoutubeReplacer {
    fn get_regex(&self) -> &Regex {
        &self.config.regex
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming Youtube URL...");
        let url = if url.contains("watch") {
            // Handle regular youtube link
            let mut url = Url::parse(url)?;
            let query = url
                .query_pairs()
                .into_iter()
                .filter_map(|(field, value)| {
                    if field == "v" {
                        Some(format!("{field}={value}"))
                    } else {
                        None
                    }
                })
                .take(1)
                .next()
                .ok_or(ReplaceError::NoQueryParams)?;
            url.set_query(Some(&query));
            url.to_string()
        } else {
            // Handle shorts link
            url.to_string()
        };
        let new_url = self
            .config
            .domain_regex
            .replace(&url, format!("{}/", self.config.new_domain))
            .to_string();
        debug! {%new_url, "new url"};
        let mut url = Url::parse(&new_url)?;
        if self.config.strip_query {
            url.set_query(None);
        };

        Ok(url.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct YoutubeConfig {
    new_domain: String,
    regex: Regex,
    domain_regex: Regex,
    strip_query: bool,
}

impl YoutubeConfig {
    pub fn new(
        new_domain: String,
        regex: String,
        domain_regex: String,
        strip_query: bool,
    ) -> ReplaceConfigResult<Self> {
        let regex = Regex::new(&regex)?;
        let domain_regex = Regex::new(&domain_regex)?;
        Ok(Self {
            new_domain,
            regex,
            domain_regex,
            strip_query,
        })
    }
}

impl TryFrom<&LinkReplacerConfig> for YoutubeConfig {
    type Error = ReplaceConfigError;
    fn try_from(value: &LinkReplacerConfig) -> Result<Self, Self::Error> {
        let new_domain = value
            .new_domain
            .clone()
            .unwrap_or(youtube_default_new_domain());
        let regex = value.regex.clone().unwrap_or(youtube_default_link_re_str());
        let domain_regex = value
            .domain_re
            .clone()
            .unwrap_or(youtube_default_domain_re_str());
        let strip_query = value.strip_query.unwrap_or(youtube_default_strip_query());
        Self::new(new_domain, regex, domain_regex, strip_query)
    }
}

impl Default for YoutubeConfig {
    fn default() -> Self {
        Self::new(
            youtube_default_new_domain(),
            youtube_default_link_re_str(),
            youtube_default_domain_re_str(),
            youtube_default_strip_query(),
        )
        .unwrap()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> YoutubeReplacer {
        YoutubeReplacer::new(YoutubeConfig::default())
    }

    #[tokio::test]
    async fn test_transform_shorts_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://youtube.com/shorts/xFnfOdb35FI/";
        let expected = "https://youtu.be/xFnfOdb35FI/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn test_transform_normal_youtube_link() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://www.youtube.com/watch?v=Z5OUviAH2Yc/";
        let expected = "https://youtu.be/Z5OUviAH2Yc/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn test_transform_normal_youtube_link_extra_query_params() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://www.youtube.com/watch?some=field&v=Z5OUviAH2Yc/";
        let expected = "https://youtu.be/Z5OUviAH2Yc/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn test_transform_mobile_youtube_links() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer();
        let url = "https://m.youtube.com/watch?v=Z5OUviAH2Yc/";
        let expected = "https://youtu.be/Z5OUviAH2Yc/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
