use crate::rss::Entry;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigEntry {
    pub name: String,
    pub url: String,
}

fn feedfile() -> PathBuf {
    let home = env::var("HOME").expect("$HOME isn't set. Wot?");
    PathBuf::from(home).join(".config/azfeed/feeds.toml")
}

pub fn read_feeds() -> Vec<ConfigEntry> {
    match fs::read_to_string(feedfile()) {
        Ok(x) => toml::from_str::<Vec<ConfigEntry>>(&x).expect("Can't parse feed file"),
        Err(e) => panic!("Implement behaviour to create feed file and launch default here"),
    }
}

pub fn write_feeds(feeds: &Vec<Entry>) -> Result<(), Box<dyn Error>> {
    // TODO: This leads to the table entries having the title of [[]], which is
    // not pretty. Find out how to give this a nice hading.
    let str = toml::to_string_pretty(feeds)?;
    fs::write(feedfile(), str)?;

    Ok(())
}
