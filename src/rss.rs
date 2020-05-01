use crate::utils;
use chrono::prelude::*;
use futures::future::join_all;
use quick_xml;
use quick_xml::events::Event;
use reqwest::Client;
use rss::{Channel, Item};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use tokio;

#[derive(Serialize, Deserialize, Debug)]
pub struct Feed {
    pub title: String,
    pub html_url: String,
    pub xml_url: String,
    pub date_of_last_read_entry: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub rss_entry: Item,
    pub html_url: String,
    pub title: String,
}

pub fn load_feeds(feeds: &Vec<Feed>) -> Vec<Channel> {
    // TODO: This might only kick off a single thread, still only fetching everything sequentially.
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let feeds = runtime.block_on(async { read_channels(feeds).await });
    feeds
        .into_iter()
        .map(|a| a.map_err(|e| eprintln!("Failure in retrieving feed: {:?}", e)))
        .filter(|c| c.is_ok())
        .map(|a| a.unwrap())
        .map(|response| {
            // This is a hack, sometimes there's an "Incomplete body" that
            // throws here. Fuck's sake.
            match runtime.block_on(response.text()) {
                Ok(text) => Some(Channel::read_from(text.as_bytes())),
                Err(e) => {
                    eprintln!("Can't acquire response body: {:?}", e);
                    None
                }
            }
        })
        .filter(|c| c.is_some())
        .map(|a| a.unwrap())
        .map(|a| match a {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Can't construct channel from response: {:?}", e);
                None
            }
        })
        .filter(|a| a.is_some())
        .map(|a| a.unwrap())
        .collect()
}

async fn read_channels(feeds: &Vec<Feed>) -> Vec<Result<reqwest::Response, reqwest::Error>> {
    let client = Client::new();
    let futures = feeds.iter().map(|entry| client.get(&entry.xml_url).send());
    join_all(futures).await
}

pub fn get_unread_entries() -> Vec<Entry> {
    let feeds = utils::read_feeds();
    let channels = load_feeds(&feeds);
    let last_read = feeds
        .iter()
        .map(|a| {
            (
                a.html_url.clone(),
                a.date_of_last_read_entry
                    .clone()
                    .unwrap_or(Local.timestamp(0, 0).to_string()),
            )
        }) // If no last-read entry, use epoch as a "hasn't ever been read")
        .map(|(url, date)| {
            (
                url,
                parse_time(&date),
            )
        })
        .collect::<HashMap<String, DateTime<Local>>>();

    channels
        .into_iter()
        .map(|feed| {
            let last_read_stamp = last_read
                .get(feed.link())
                .unwrap_or_else(|| panic!("Last-read dict does not have entry for this channel. Check http/https in html_url. feed: {:?}", feed));
            feed.items()
                .iter()
                .cloned()
                .filter(|item| publication_date(item).unwrap_or(Local::now()) > *last_read_stamp)
                .collect::<Vec<rss::Item>>()
        })
        .map(|vec| {
            vec.into_iter()
                .map(|item| Entry {
                    title: item.title().unwrap_or("").to_string(),
                    html_url: item.link().unwrap_or("").to_string(),
                    rss_entry: item,
                })
                .collect::<Vec<Entry>>()
        })
        .flatten()
        .collect::<Vec<Entry>>()
}

fn publication_date(item: &rss::Item) -> Option<DateTime<Local>> {
    if let Some(x) = item.pub_date() {
        return Some(parse_time(x))
    };

    if let Some(x) = item.dublin_core_ext() {
        match x.dates() {
            [date] => {
                return Some(parse_time(date))
            }
            _ => {},
        }
    };

    None
}

fn parse_time(time: &str) -> DateTime<Local> {
    match time.parse::<DateTime<Local>>() {
        Ok(x) => x,
        Err(_) => match DateTime::parse_from_rfc2822(time) {
            Ok(x) => DateTime::from(x),
            Err(_) => match DateTime::parse_from_rfc3339(time) {
                Ok(x) => DateTime::from(x),
                Err(_) => match DateTime::parse_from_rfc2822(&format!("{} +0000", time)) {
                    Ok(x) => DateTime::from(x),
                    Err(e) => panic!("Can't parse date from common formats: {:?}", e)
                }
            }
        }
    }
}

pub fn mark_as_read(read_entry: &Entry) -> Result<(), Box<dyn Error>> {
    let mut feeds = utils::read_feeds();
    let entry_date = read_entry
        .rss_entry
        .pub_date()
        .expect("Read Entry doesn't have a publication date");
    let entry_date = parse_time(entry_date);

    feeds
        .iter_mut()
        .filter(|f| f.html_url == read_entry.html_url)
        .for_each(|feed| match &feed.date_of_last_read_entry {
            Some(date) => {
                let feed_date = parse_time(date);

                if entry_date > feed_date {
                    feed.date_of_last_read_entry = Some(entry_date.to_string())
                }
            }
            None => feed.date_of_last_read_entry = Some(entry_date.to_string()),
        });

    utils::write_feeds(feeds)
}

pub fn import_opml(path: &Path) -> Result<(), Box<dyn Error>> {
    let opml_content = fs::read_to_string(path)?;
    let mut reader = quick_xml::Reader::from_str(&opml_content);

    let mut entries = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            // The contents are in empty tags, the `Start` tags are categories.
            Ok(Event::Empty(e)) => match e.name() {
                // Potential issue: The thing we really care about here is
                // type="rss", which we currently don't check for. We just
                // assume that any empty tag has the contents we want from it.
                b"outline" => {
                    let current_entry = e
                        .attributes()
                        .map(|a| {
                            let attr = a.unwrap();
                            let value_str = String::from_utf8(attr.value.to_vec())
                                .expect("OPML is not valid utf8");
                            let key_str = String::from_utf8(attr.key.to_vec())
                                .expect("OPML is not valud utf8");
                            (key_str, value_str)
                        })
                        .collect::<Vec<_>>();
                    let mut entry = Feed {
                        title: String::new(),
                        html_url: String::new(),
                        xml_url: String::new(),
                        date_of_last_read_entry: None, // This won't be in any OPML.
                    };
                    for (key, value) in current_entry.iter() {
                        match key.as_str() {
                            "title" => entry.title = value.to_string(),
                            "xmlUrl" => entry.xml_url = value.to_string(),
                            "htmlUrl" => entry.html_url = value.to_string(),
                            _ => continue,
                        }
                    }

                    entries.push(entry);
                }
                _ => continue,
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!(e),
            _ => continue,
        }

        buf.clear()
    }

    utils::write_feeds(entries)
}
