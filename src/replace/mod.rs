use std::str::FromStr;

pub use crate::{LinkReplacerConfig, ReplacerConfig};
pub(crate) use base::{LinkProcessor, LinkReplacer, ReplaceError, ReplaceResult};
use fancy_regex::{Captures, Regex};
use strum::EnumString;
use tracing::{info, instrument, warn};

mod base;
mod bsky;
mod instagram;
mod pixiv;
mod reddit;
mod reddit_media;
mod tiktok;
mod twitter;
mod youtube;

use bsky::BskyReplacer;
use instagram::InstagramReplacer;
use pixiv::PixivReplacer;
use reddit::RedditReplacer;
use reddit_media::RedditMediaReplacer;
use tiktok::TikTokReplacer;
use twitter::TwitterReplacer;
use youtube::YoutubeReplacer;

#[derive(Debug, EnumString, PartialEq)]
enum ReplacerType {
    #[strum(ascii_case_insensitive)]
    Bsky,
    #[strum(ascii_case_insensitive)]
    Instagram,
    #[strum(ascii_case_insensitive)]
    Pixiv,
    #[strum(ascii_case_insensitive)]
    Reddit,
    #[strum(ascii_case_insensitive)]
    TikTok,
    #[strum(ascii_case_insensitive)]
    Twitter,
    #[strum(ascii_case_insensitive)]
    Youtube,
}

type BoxedLinkReplacer = Box<dyn LinkReplacer + 'static + Sync + Send>;

impl ReplacerType {
    pub fn create_type(&self, config: &LinkReplacerConfig) -> ReplaceResult<BoxedLinkReplacer> {
        let replacer: BoxedLinkReplacer = match self {
            Self::Bsky => Box::new(BskyReplacer::new(config)?),
            Self::Instagram => Box::new(InstagramReplacer::new(config)?),
            Self::Pixiv => Box::new(PixivReplacer::new(config)?),
            Self::Reddit => Box::new(RedditReplacer::new(config)?),
            Self::TikTok => Box::new(TikTokReplacer::new(config)?),
            Self::Twitter => Box::new(TwitterReplacer::new(config)?),
            Self::Youtube => Box::new(YoutubeReplacer::new(config)?),
        };
        Ok(replacer)
    }
}

static HTTP_URL_RE: &str =
    r"(?:https?://)?(?:[a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}(?:/[^\s*~`|>\[\]#()]*)?";

pub struct MessageProcessor {
    url_processors: Vec<BoxedLinkReplacer>,
    http_url_regex: Regex,
}

impl MessageProcessor {
    pub fn new(config: &ReplacerConfig, reddit_media_re: Option<String>) -> Self {
        let http_url_regex = Regex::new(HTTP_URL_RE).unwrap();
        let mut url_processors: Vec<BoxedLinkReplacer> = Vec::new();
        if let Ok(reddit_media_replacer) = RedditMediaReplacer::new(reddit_media_re)
            .map(Box::new)
            .map_err(|err| warn! {%err, "error creating reddit media replacer"})
        {
            url_processors.push(reddit_media_replacer)
        }
        for (replacer_name, config) in config.iter() {
            let new_replacer = if let Ok(replacer) = ReplacerType::from_str(replacer_name) {
                info!("Creating {} replacer...", &replacer_name);
                replacer.create_type(config)
            } else {
                Self::create_custom_replacer(replacer_name, config)
            }
            .map_err(|reason| warn! {%reason, "creating replacer"});
            if let Ok(new_replacer) = new_replacer {
                url_processors.push(new_replacer);
            }
        }
        Self {
            http_url_regex,
            url_processors,
        }
    }

    fn create_custom_replacer(
        name: &str,
        config: &LinkReplacerConfig,
    ) -> ReplaceResult<BoxedLinkReplacer> {
        if let (Some(regex), Some(domain_re), Some(strip_query)) = (
            config.regex.as_deref(),
            config.domain_re.as_deref(),
            config.strip_query,
        ) {
            info!("Creating custom replacer {}...", name);
            let custom_replacer: BoxedLinkReplacer = Box::new(LinkProcessor::new(
                &config.new_domain,
                regex,
                domain_re,
                strip_query,
            )?);
            Ok(custom_replacer)
        } else {
            Err(ReplaceError::InvalidReplacer(name.to_string()))
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn process_message(&self, msg: &str) -> (String, bool) {
        let new_msg = self
            .http_url_regex
            .replace_all(msg, |caps: &Captures<'_>| {
                self.url_processors
                    .iter()
                    .fold(caps[0].to_string(), |acc, processor| {
                        processor.process_url(&acc)
                    })
            })
            .to_string();
        let modified = new_msg.ne(msg);
        (new_msg, modified)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_processor() -> ReplaceResult<MessageProcessor> {
        let mut config = ReplacerConfig::new();
        config.insert(
            "tiktok".into(),
            LinkReplacerConfig::new("vxtiktok.com".into()),
        );
        config.insert("youtube".into(), LinkReplacerConfig::new("youtu.be".into()));
        let processor = MessageProcessor::new(&config, None);
        Ok(processor)
    }

    #[tokio::test]
    async fn test_message_replace() -> ReplaceResult<()> {
        let processor = create_processor()?;
        let message =
            "Test message with a TikTok link ||https://www.tiktok.com/t/ZTYXjHYeg/|| in it.";
        let expected =
            "Test message with a TikTok link ||https://www.vxtiktok.com/t/ZTYXjHYeg/|| in it.";

        let (result, modified) = processor.process_message(message);
        assert!(modified);
        assert_eq!(&result, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_message_replace() -> ReplaceResult<()> {
        let processor = create_processor()?;
        let message = "Test message with **multiple** (https://www.tiktok.com/t/ZTYX2qUvY/) types of links ||https://youtube.com/shorts/xFnfOdb35FI/|| in it.";
        let expected = "Test message with **multiple** (https://www.vxtiktok.com/t/ZTYX2qUvY/) types of links ||https://youtu.be/xFnfOdb35FI/|| in it.";

        let (result, modified) = processor.process_message(message);
        assert!(modified);
        assert_eq!(&result, expected);
        Ok(())
    }
}
