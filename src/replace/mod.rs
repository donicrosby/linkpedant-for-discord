use std::str::FromStr;

pub use crate::{LinkReplacerConfig, ReplacerConfig};
pub(crate) use base::{
    LinkProcessor, LinkReplacer, ProcessorConfig, ReplaceConfigError, ReplaceConfigResult,
    ReplaceError, ReplaceResult,
};
use fancy_regex::{Captures, Regex};
use strum::EnumString;
use tracing::{info, instrument, warn};

mod amazon;
mod base;
mod bsky;
mod instagram;
mod pixiv;
mod reddit;
mod reddit_media;
mod tiktok;
mod twitter;
mod youtube;

use amazon::AmazonReplacer;
pub(super) use bsky::BskyReplacer;
pub(super) use instagram::InstagramReplacer;
pub(super) use pixiv::PixivReplacer;
pub(super) use reddit::RedditReplacer;
use reddit_media::RedditMediaReplacer;
pub(super) use tiktok::TikTokReplacer;
pub(super) use twitter::TwitterReplacer;
pub(super) use youtube::YoutubeReplacer;

pub use amazon::AmazonConfig;

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
    pub fn create_type(
        &self,
        config: &LinkReplacerConfig,
    ) -> ReplaceConfigResult<BoxedLinkReplacer> {
        let replacer: BoxedLinkReplacer = match self {
            Self::Bsky => Box::new(BskyReplacer::new(config.try_into()?)),
            Self::Instagram => Box::new(InstagramReplacer::new(config.try_into()?)),
            Self::Pixiv => Box::new(PixivReplacer::new(config.try_into()?)),
            Self::Reddit => Box::new(RedditReplacer::new(config.try_into()?)),
            Self::TikTok => Box::new(TikTokReplacer::new(config.try_into()?)),
            Self::Twitter => Box::new(TwitterReplacer::new(config.try_into()?)),
            Self::Youtube => Box::new(YoutubeReplacer::new(config.try_into()?)),
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
    pub fn new(
        config: &ReplacerConfig,
        reddit_media_re: Option<String>,
        amazon_config: &AmazonConfig,
    ) -> Self {
        let http_url_regex = Regex::new(HTTP_URL_RE).unwrap();
        let mut url_processors: Vec<BoxedLinkReplacer> = Vec::new();
        if let Ok(reddit_media_replacer) = RedditMediaReplacer::new(reddit_media_re)
            .map(Box::new)
            .map_err(|err| warn! {%err, "error creating reddit media replacer"})
        {
            url_processors.push(reddit_media_replacer)
        }
        if let Ok(amazon_replacer) = AmazonReplacer::new(amazon_config)
            .map(Box::new)
            .map_err(|err| warn! {%err, "error creating amazon shortener"})
        {
            url_processors.push(amazon_replacer)
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
    ) -> ReplaceConfigResult<BoxedLinkReplacer> {
        if let Some(new_domain) = config.new_domain.clone() {
            if let (Some(regex), Some(domain_re), Some(strip_query)) = (
                config.regex.as_deref(),
                config.domain_re.as_deref(),
                config.strip_query,
            ) {
                info!("Creating custom replacer {}...", name);
                let config = ProcessorConfig::new(new_domain, regex, domain_re, strip_query)?;
                let custom_replacer: BoxedLinkReplacer = Box::new(LinkProcessor::new(config));
                Ok(custom_replacer)
            } else if config.regex.is_none() {
                Err(ReplaceConfigError::MissingOption("Link Regex".to_string()))
            } else if config.domain_re.is_none() {
                Err(ReplaceConfigError::MissingOption(
                    "Domain Regex".to_string(),
                ))
            } else {
                Err(ReplaceConfigError::MissingOption("Strip Query".to_string()))
            }
        } else {
            Err(ReplaceConfigError::InvalidReplacer(name.to_string()))
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn process_message(&self, msg: &str) -> ReplaceResult<Option<String>> {
        let mut could_modify = false;
        let mut process_error = None;
        let new_msg = self
            .http_url_regex
            .replace_all(msg, |caps: &Captures<'_>| {
                self.url_processors
                    .iter()
                    .fold(caps[0].to_string(), |acc, processor| {
                        match processor.process_url(&acc) {
                            Ok(Some(new_url)) => {
                                could_modify = true;
                                new_url
                            }
                            Ok(None) => acc,
                            Err(err) => {
                                process_error = Some(err);
                                acc
                            }
                        }
                    })
            })
            .to_string();
        if let Some(process_err) = process_error {
            Err(process_err)
        } else if could_modify {
            Ok(Some(new_msg))
        } else {
            Ok(None)
        }
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
        let processor = MessageProcessor::new(&config, None, &AmazonConfig::default());
        Ok(processor)
    }

    #[tokio::test]
    async fn test_message_replace() -> ReplaceResult<()> {
        let processor = create_processor()?;
        let message =
            "Test message with a TikTok link ||https://www.tiktok.com/t/ZTYXjHYeg/|| in it.";
        let expected =
            "Test message with a TikTok link ||https://www.vxtiktok.com/t/ZTYXjHYeg/|| in it.";

        let result = processor.process_message(message);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        assert_eq!(&result.unwrap(), expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_message_replace() -> ReplaceResult<()> {
        let processor = create_processor()?;
        let message = "Test message with **multiple** (https://www.tiktok.com/t/ZTYX2qUvY/) types of links ||https://youtube.com/shorts/xFnfOdb35FI/|| in it.";
        let expected = "Test message with **multiple** (https://www.vxtiktok.com/t/ZTYX2qUvY/) types of links ||https://youtu.be/xFnfOdb35FI/|| in it.";

        let result = processor.process_message(message);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        assert_eq!(&result.unwrap(), expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_unknown_link_types() -> ReplaceResult<()> {
        let processor = create_processor()?;
        let message = "Test message with [unknown link](https://example.com/v/Fsd6ZMcG0XN6OmOK) does not result in an error";

        let result = processor.process_message(message);
        assert!(result.is_ok());
        let result: Option<String> = result.unwrap();
        assert!(result.is_none());
        Ok(())
    }
}
