use super::{LinkReplacer, ReplaceConfigResult, ReplaceError, ReplaceResult};
use fancy_regex::Regex;
use serde::Deserialize;
use tracing::{debug, instrument, warn};

#[derive(Debug, Clone)]
pub struct AmazonReplacer {
    shorten_domain: String,
    regex: Regex,
    shorten: bool,
}

const AMAZON_LINK_RE_STR: &str =
    r"https?://(www\.)?amazon\.com/(?:[^\s]+/)?(dp/(?<asin>\w+))/[^\s]+";
const AMAZON_SHORT_DOMAIN: &str = "amzn.com";

fn amazon_default_re_str() -> String {
    AMAZON_LINK_RE_STR.to_string()
}

fn amazon_shorten_domain() -> String {
    AMAZON_SHORT_DOMAIN.to_string()
}

fn amazon_shorten() -> bool {
    false
}

#[derive(Debug, Deserialize)]
pub struct AmazonConfig {
    #[serde(default = "amazon_default_re_str")]
    pub regex: String,
    #[serde(default = "amazon_shorten_domain")]
    pub shorten_domain: String,
    #[serde(default = "amazon_shorten")]
    pub shorten: bool,
}

impl Default for AmazonConfig {
    fn default() -> Self {
        let regex = amazon_default_re_str();
        let shorten_domain = amazon_shorten_domain();
        let shorten = amazon_shorten();
        Self {
            regex,
            shorten_domain,
            shorten,
        }
    }
}

impl AmazonReplacer {
    pub fn new(config: &AmazonConfig) -> ReplaceConfigResult<Self> {
        let shorten_domain = config.shorten_domain.clone();
        let regex = Regex::new(&config.regex)?;
        let shorten = config.shorten;

        Ok(Self {
            shorten_domain,
            regex,
            shorten,
        })
    }
}

impl LinkReplacer for AmazonReplacer {
    fn get_regex(&self) -> &Regex {
        &self.regex
    }

    #[instrument(skip(self))]
    fn transform_url(&self, url: &str) -> ReplaceResult<String> {
        let url = if let Ok(Some(caps)) = self
            .regex
            .captures(url)
            .map_err(|err| warn! {%err, "error processing amazon link"})
        {
            debug!("Shortening Amazon URL...");
            let asin = caps
                .name("asin")
                .ok_or(ReplaceError::MissingGroup("asin".to_string()))
                .inspect_err(|_| {
                    warn!("No asin capture group name defined! Cannot process link...");
                })
                .map(|a| a.as_str())?;
            if self.shorten {
                format!(
                    "https://{short_domain}/dp/{asin}/",
                    short_domain = self.shorten_domain
                )
            } else {
                format!("https://www.amazon.com/dp/{asin}/")
            }
        } else {
            url.to_string()
        };
        Ok(url)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::init_tests;

    fn create_test_replacer(shorten: bool) -> ReplaceConfigResult<AmazonReplacer> {
        AmazonReplacer::new(&AmazonConfig {
            shorten,
            ..Default::default()
        })
    }

    #[tokio::test]
    async fn test_transform_url() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer(false)?;
        let url = "https://www.amazon.com/Gears-Wonderland-steampunk-fantasy-Anderson-ebook/dp/B005USJ5U8/ref=sr_1_1?ie=UTF8&qid=1491136398&sr=8-1&keywords=gears+of+wonderland";
        let expected = "https://www.amazon.com/dp/B005USJ5U8/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn test_transform_to_shortened() -> ReplaceResult<()> {
        init_tests().await;
        let test_replacer = create_test_replacer(true)?;
        let url = "https://www.amazon.com/Gears-Wonderland-steampunk-fantasy-Anderson-ebook/dp/B005USJ5U8/ref=sr_1_1?ie=UTF8&qid=1491136398&sr=8-1&keywords=gears+of+wonderland";
        let expected = "https://amzn.com/dp/B005USJ5U8/";

        let result = test_replacer.transform_url(&url)?;
        assert_eq!(expected, result);
        Ok(())
    }
}
