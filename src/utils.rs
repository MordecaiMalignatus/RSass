use crate::rss::Feed;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Struct to give the feeds.toml a nice "feed" headline in the array-table
/// entry, rather than `[[]]`.
#[derive(Debug, Serialize, Deserialize)]
struct Feeds {
    feed: Vec<Feed>,
}

fn feedfile() -> PathBuf {
    let home = env::var("HOME").expect("$HOME isn't set. Wot?");
    PathBuf::from(home).join(".config/azfeed/feeds.toml")
}

pub fn read_feeds() -> Vec<Feed> {
    match fs::read_to_string(feedfile()) {
        Ok(x) => {
            toml::from_str::<Feeds>(&x)
                .expect("Can't parse feed file")
                .feed
        }
        Err(_e) => panic!("Implement behaviour to create feed file and launch default here"),
    }
}

pub fn write_feeds(feeds: Vec<Feed>) -> Result<(), Box<dyn Error>> {
    let str = toml::to_string_pretty(&Feeds { feed: feeds })?;
    fs::write(feedfile(), str)?;

    Ok(())
}
