use crate::utils;
use quick_xml;
use quick_xml::events::Event;
use rss::{Channel, ChannelBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub title: String,
    pub html_url: String,
    pub xml_url: String,
}

// TODO: This'll need parallelisation. Sequential is unbearably slow. Perhaps
// even caching.
pub fn load_feeds(feeds: &Vec<Entry>) -> Vec<Channel> {
    feeds
        .iter()
        .map(|a| match Channel::from_url(&a.xml_url) {
            Ok(mut c) => {
                c.set_link(&a.html_url);
                c.set_title(&a.title);
                Some(c)
            }
            Err(_) => None,
        })
        .filter(|c| c.is_some())
        .map(|c| c.unwrap())
        .collect()
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
