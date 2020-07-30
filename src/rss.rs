use crate::utils;
use chrono::prelude::*;
use futures::future::join_all;
use quick_xml;
use quick_xml::events::Event;
use reqwest::Client;
use rss::{Channel, Item};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;
use tokio;

#[derive(Serialize, Deserialize, Debug)]
pub struct Feed {
    pub title: String,
    pub html_url: String,
    pub xml_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub rss_entry: Item,
    pub html_url: String,
    pub title: String,
    pub feed: String,
}

pub fn load_feeds(feeds: Vec<Feed>) -> Vec<(Feed, Channel)> {
    // TODO: This might only kick off a single thread, still only fetching everything sequentially.
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let channels = runtime.block_on(async { read_channels(&feeds).await });
    let mut res = Vec::new();

    channels
        .into_iter()
        .zip(feeds.into_iter())
        .for_each(|(channel, feed)| match channel {
            Err(e) => eprintln!("Failure in retrieving feed {}: {:?}", feed.title, e),
            Ok(x) => match runtime.block_on(x.text()) {
                Err(e) => eprintln!(
                    "Can't acquire response body for feed {}: {:?}",
                    feed.title, e
                ),
                Ok(text) => match Channel::read_from(text.as_bytes()) {
                    Ok(c) => res.push((feed, c)),
                    Err(e) => eprintln!(
                        "Can't construct channel from response for feed {}: {:?}",
                        feed.title, e
                    ),
                },
            },
        });

    res
}

async fn read_channels(feeds: &Vec<Feed>) -> Vec<Result<reqwest::Response, reqwest::Error>> {
    let client = Client::new();
    let futures = feeds.iter().map(|entry| client.get(&entry.xml_url).send());
    join_all(futures).await
}

pub fn get_unread_entries() -> Vec<Entry> {
    unimplemented!();
    // let feeds = utils::read_feeds();
    // let feeds_and_channels = load_feeds(feeds);

    // let mut res = Vec::new();

    // for (feed, channel) in feeds_and_channels {
    //     let items = channel.items();
    //     let empty_read = Vec::new();
    //     let feed_cache = dbg!(cache.get(&feed.title).unwrap_or(&empty_read));

    //     items
    //         .into_iter()
    //         .filter(|item| match item.guid() {
    //             Some(x) => feed_cache.contains(&x.value().to_string()),
    //             None => true,
    //         })
    //         .map(|item| Entry {
    //             title: feed.title.clone(),
    //             rss_entry: item.clone(),
    //             html_url: feed.html_url.clone(),
    //             feed: feed.xml_url.clone(),
    //         })
    //         .for_each(|item| res.push(item))
    // }

    // res.reverse();
    // res
}

fn publication_date(item: &rss::Item) -> Option<DateTime<Local>> {
    if let Some(x) = item.pub_date() {
        return Some(parse_time(x));
    };

    if let Some(x) = item.dublin_core_ext() {
        match x.dates() {
            [date] => return Some(parse_time(date)),
            _ => {}
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
                // Sometimes, timestamps are in RFC-2822 format, but lack the
                // timezone specifier, thus making it technically invalid. Add
                // neutral offset, then try again, then see if it parses.
                Err(_) => match DateTime::parse_from_rfc2822(&format!("{} +0000", time)) {
                    Ok(x) => DateTime::from(x),
                    Err(e) => panic!("Can't parse date from common formats: {:?}", e),
                },
            },
        },
    }
}

pub fn mark_as_read(read_entry: &Entry) -> Result<(), Box<dyn Error>> {
    unimplemented!()
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
