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
  - TikTok (video): via [fxtiktok](https://github.com/okdargy/fxtiktok)
  - Instagram (image, video, and reels): via [ddinstagram](https://github.com/Wikidepia/InstaFix)
  - Reddit (text, image, and video): via [vxreddit](https://github.com/dylanpdx/vxReddit)
  - YouTube (shorts and normal videos): via a `youtu.be` URL that will link to a full player

It also supports custom sites as long as you just need to swap the domain (i.e. some-site.com -> fxsome-site.com) by just adding them to the list of replacers.

Just modify your config like so
```
replacers:
    ...
    some_site:
        new_domain: "fxsome-site.com" # replacement domain
        regex: https?://(\w+\.)?some-site\.com/[^\s]+ # regex for what URLs that should be modified
        domain_re: (\w+\.)?(some-site\.com) # regex for 
        strip_query: true # You should probably strip the query string
    other_replacer:
        ...
```

## Current commands
- `/help`: Displays help message with list of commands
- `/invite`: Get a link to invite the bot to your server

## Self-Hosting

Setting up your own instance of the bot is pretty straightforward:

- Clone the repository or download the `docker-compose.yaml`
- Create your own copy of the config file
  - `cp config.example.yaml config.yaml`
- Update the config with your Discord Bot Token (see the Discord Docs for how)
- Once you run the bot the logs will output the proper link you need to go to in order to add your bot to your server