# LinkPedant for Discord

## What is LinkPedant?

LinkPedant fixes link embeds in messages sent in your server!

Media embeds on Discord can be hit or miss, sometimes they work or (usually) they don't.

Most social media wants users interacting directly with their site (a key "metric" used) not through some embedding somewhere else, so they usually don't make it easy to embed the posted content elsewhere.

This bot fixes those embeds so that you don't have to leave the server to view whatever content someone has linked to.

## Supported Sites
Currently LinkPedant supports the following sites:
    - Twitter: via [fxtwitter](https://github.com/FixTweet/FixTweet)
    - Bluesky: via [vixbluesky](https://github.com/Rapougnac/VixBluesky)
    - TikTok (video): via [vxtiktok](https://github.com/dylanpdx/vxtiktok)
    - Instagram (image, video, and reels): via [ddinstagram](https://github.com/Wikidepia/InstaFix)
    - Reddit (text, image, and video): via [vxreddit](https://github.com/dylanpdx/vxReddit)
    - YouTube Shorts: via a `youtu.be` URL that will link to a full player

It also supports custom sites as long as you just need to swap the domain (i.e. some-site.com -> fxsome-site.com) by just adding them to the list of replacers.

Just modify your config like so
```
replacers:
    ...
    some_site:
        new_domain: "fxsome-site.com" # replacement domain
        regex: https?://(\w+\.)?some-site\.com/[^\s]+ # regex for what URLs that should be modified
        domain_re: (\w+\.)?(some-site\.com) # regex for 
        strip_query: true
    other_replacer:
        ...
```

## Current commands
- `/help`: Displays help message with list of commands
- `/invite`: Get a link to invite the bot to your server

## Self-Hosting


A bot that will fix Discord Embeddings for various links 
#     # Custom URL matching regex
#     regex: https?://(\w+\.)?instagram.com/(p|reel|stories)/[^\s]+
#     # Custom Domain replacement regex
#     domain_re: (\w+\.)?(instagram\.com)
#     # Should strip query string
#     strip_query: bool

Test