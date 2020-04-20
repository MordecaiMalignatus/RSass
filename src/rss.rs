use crate::utils;
use quick_xml;
use reqwest::Client;
use quick_xml::events::Event;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;
use futures::future::join_all;
use tokio;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub title: String,
    pub html_url: String,
    pub xml_url: String,
}

pub fn load_feeds(feeds: &Vec<Entry>) -> Vec<Channel> {
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let feeds = runtime.block_on(async { read_channels(feeds).await });
    feeds.into_iter()
        .map(|a| a.map_err(|e| eprintln!("Failure in retrieving feed: {:?}", e)))
        .filter(|c| c.is_ok())
        .map(|a| a.unwrap())
        .map(|response| {
            // This is a hack, sometimes there's an "Incomplete body" that
            // throws here. Fuck's sake.
            let text = runtime.block_on(response.text()).unwrap_or(String::new());
            Channel::read_from(text.as_bytes())
        })
        .filter(|c| c.is_ok())
        .map(|a| a.unwrap())
        .collect()

}

async fn read_channels(feeds: &Vec<Entry>) -> Vec<Result<reqwest::Response, reqwest::Error>> {
    let client = Client::new();
    let futures = feeds.iter().map(|entry| {
        client.get(&entry.xml_url).send()
    });
    join_all(futures).await
}

pub fn get_unread_entries() -> Vec<Entry> {
    unimplemented!()
}

pub fn mark_as_read(read_entry: &Entry) -> () {
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
                    let mut entry = Entry {
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
