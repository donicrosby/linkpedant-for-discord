use super::{LinkReplacer, ReplaceError, ReplaceResult};
use fancy_regex::Regex;
use tracing::{debug, instrument};
use url::Url;
use urlencoding::decode;

#[derive(Debug, Clone)]
pub struct RedditMediaReplacer {
    regex: Regex,
}

const REDDIT_MEDIA_LINK_RE_STR: &'static str = r"https?://(\w+\.)?reddit\.com/media[^\s]+";

const FIXABLE_TYPES: &[&'static str] = &[".jpeg", ".jpg", ".png", ".gif", ".webp"];

impl RedditMediaReplacer {
    pub fn new(media_re_str: Option<String>) -> ReplaceResult<Self> {
        let regex_str = media_re_str.unwrap_or(REDDIT_MEDIA_LINK_RE_STR.to_string());
        let regex = Regex::new(&regex_str)?;
        Ok(Self { regex })
    }

    fn fixable_url(&self, url: &str) -> bool {
        FIXABLE_TYPES.iter().any(|f_type| url.contains(*f_type))
    }
}

impl LinkReplacer for RedditMediaReplacer {
    fn get_regex(&self) -> &Regex {
        &self.regex
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        debug!("Transforming Reddit Media URL...");
        let url = Url::parse(url)?;
        let media_url = url
            .query_pairs()
            .into_iter()
            .filter(|(param, _val)| param == "url")
            .map(|(_, val)| val)
            .reduce(|acc, _| acc)
            .ok_or_else(|| ReplaceError::NoQueryParams)?;
        let decoded = decode(&media_url).map_err(|_| ReplaceError::Utf8Decode)?;
        if self.fixable_url(&decoded) {
            let adjusted_url = decoded.replace("//preview.", "//i.");
            let mut new_url = Url::parse(&adjusted_url)?;
            new_url.set_query(None);
            Ok(new_url.to_string())
        } else {
            Ok(decoded.to_string())
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer() -> ReplaceResult<RedditMediaReplacer> {
        RedditMediaReplacer::new(None)
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer()?;
        let url = "https://www.reddit.com/media?url=https%3A%2F%2Fpreview.redd.it%2Ffor-those-who-try-to-have-a-main-in-each-class-who-are-your-v0-8uo8tgdfb08e1.jpeg%3Fwidth%3D640%26crop%3Dsmart%26auto%3Dwebp%26s%3Daff0061f8f21aec6bdb13a4811c8978ae2f5fd9c";
        let expected = "https://i.redd.it/for-those-who-try-to-have-a-main-in-each-class-who-are-your-v0-8uo8tgdfb08e1.jpeg";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
