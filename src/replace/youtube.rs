use super::{LinkReplacer, LinkReplacerConfig, ReplaceError, ReplaceResult};
use fancy_regex::Regex;
use tracing::{debug, instrument};
use url::Url;

#[derive(Debug, Clone)]
pub struct YoutubeReplacer {
    new_domain: String,
    regex: Regex,
    domain_regex: Regex,
    strip_query: bool,
}

const YOUTUBE_LINK_RE_STR: &str =
    r"https?://((www|m)\.)?youtube\.com/(shorts/[^\s]+|watch\?(?:&?(?:[^\s]+\=[^\s]+))+)";
const YOUTUBE_DOMAIN_RE_STR: &str = r"((www|m)\.)?(youtube\.com/(shorts/|watch\?v=))";

impl YoutubeReplacer {
    pub fn new(config: &LinkReplacerConfig) -> ReplaceResult<Self> {
        let new_domain = config.new_domain.to_owned();
        let regex = Regex::new(config.regex.as_deref().unwrap_or(YOUTUBE_LINK_RE_STR))?;
        let domain_regex =
            Regex::new(config.domain_re.as_deref().unwrap_or(YOUTUBE_DOMAIN_RE_STR))?;
        let strip_query = config.strip_query.unwrap_or(true);

        Ok(Self {
            new_domain,
            regex,
            domain_regex,
            strip_query,
        })
    }
}

impl LinkReplacer for YoutubeReplacer {
    fn get_regex(&self) -> &Regex {
        &self.regex
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
            .domain_regex
            .replace(&url, format!("{}/", self.new_domain))
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
    async fn test_transform_shorts_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://youtube.com/shorts/xFnfOdb35FI/";
        let expected = "https://youtu.be/xFnfOdb35FI/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn test_transform_normal_youtube_link() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://www.youtube.com/watch?v=Z5OUviAH2Yc/";
        let expected = "https://youtu.be/Z5OUviAH2Yc/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn test_transform_normal_youtube_link_extra_query_params() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://www.youtube.com/watch?some=field&v=Z5OUviAH2Yc/";
        let expected = "https://youtu.be/Z5OUviAH2Yc/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn test_transform_mobile_youtube_links() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://m.youtube.com/watch?v=Z5OUviAH2Yc/";
        let expected = "https://youtu.be/Z5OUviAH2Yc/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
