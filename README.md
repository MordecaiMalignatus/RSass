# RSass

RSass is a small RSS reader that was written out of annoyance with Feedly.

It does not have a lot of features, it is just a tiny, hacked-together thing,
and it barely works. But it does work!

**WARNING**: This is alpha quality software. Expect bugs, stuff to break, things
to not work properly. Bug reports are appreciated, and RSass will get
better. Until then, you have been warned.

![Screenshot](./.documentation/RSass-preview.png)

## Getting Started

1. Download the Repo
2. Build: `cargo build --release`.
3. Put the release binary somewhere where you can use it.
4. Either:
   - Use the [OPML import](#opml_import) to import your existing feed list or
   - Edit your `~/.config/rsass/feeds.toml` to include the feeds you want. More
     on that [here](#feeds_toml)
5. Run `rsass`. Right now, this will display nothing, fetch everything, and then
   require you to restart `rsass` to read the unread entries from the DB. This
   will be fixed to work as expected in the future.

### The `feeds.toml`.

<a name="feeds_toml"></a>
The only way of configuring RSass, this is a TOML file following a simple
format.

You add a feed with an entry like this:

```toml
[[feed]]
title = "A Blag on the Intertubes"
html_url = "https://blog.xkcd.com"
xml_url = "https://blog.xkcd.com/feed/"
```

Right now, you have to specify all keys by hand, no auto-discovery is
supported. To explain the values:

- The title right now is mostly for you to name your feeds. In the future, this
  will also be used for display purposes.
- The HTML URL is what will be opened in a browser if you click the "open in
  Browser" button.
- The XML URL is what is fetched by RSass and its contents are displayed to
  you.

## Additional niceties.

### OPML import

<a name="opml_import"></a>
The easiest way to try out RSass is to import an OPML bundle that you can get
from Feedly or similar services. Importing your OPML to
`~/.config/rsass/feeds.toml` is as simple as building RSass, and running `rsass
import your_opml_bundle.xml`. This will write to the [`feeds.toml`](#feeds_toml)
where you can edit further.

## Core concepts

### File-based

Like many stalwarts of *nix, this is entirely controlled via a file, in this
case `~/.rsass/feeds.toml`. Everything is kept in this file, and you add more
feeds by modifying that file.

### No servers or daemons.

Feeds are fetched on startup. This may eventually change, simply because it's
slow as hell.

### No selective reading.

This is conceived as a "reading queue" of RSS -- You read all of it, one article
after the next. There is no selection list, and it assumes that you want to
consume all of the content in your RSS feeds.

# Contributing

I welcome any patch to make this thing better, because I do want to use it as
primary RSS reader. If you're not sure what to work on, file an issue with
something that annoys you about using it, and we'll chat about the best way to
fix it.

Licensed under the GPL v3.
