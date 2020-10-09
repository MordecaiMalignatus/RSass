use rusqlite::{params, Connection};

use crate::{rss::Entry, utils};
use std::error::Error;
use std::path::PathBuf;

pub fn get_db_connection() -> Connection {
    get_connection(utils::db_path())
}

fn get_connection(path: PathBuf) -> Connection {
    match Connection::open(&path) {
        Ok(c) => {
            create_tables(&c).expect("Can't create tables in connection creation");
            c
        }
        Err(e) => {
            panic!("Can't open DB file: {}", e);
        }
    }
}

fn create_tables(conn: &Connection) -> Result<usize, rusqlite::Error> {
    conn.execute(
        r#"
CREATE TABLE IF NOT EXISTS entries(
  title text,
  content text,
  read integer,
  feed text,
  guid blob,
  html_url text
);
"#,
        params![],
    )
}

pub fn mark_entry_as_read(conn: &Connection, guid: &String) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn
        .prepare("update entries set read = 1 where guid = ?")
        .expect("Can't prepare mark_read query");
    match stmt.execute(params![guid]) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Can't mark item with guid {}, reason: {}", guid, e);
            Err(Box::new(e))
        }
    }
}

pub fn insert_new_unread_entries(conn: &Connection, items: Vec<Entry>) -> () {
    let mut stmt = conn
        .prepare("insert into entries(title, content, read, feed, guid, html_url) values (?, ?, 0, ?, ?, ?)")
        .expect("Can't prepare statement for inserting unread entries.");

    items.into_iter().for_each(|entry| {
        match stmt.execute(params![
            entry.title,
            entry.content,
            entry.feed,
            entry.guid,
            entry.html_url,
        ]) {
            Err(e) => eprintln!("Coulnd't insert entry into DB: {}\n{:#?}", e, entry),
            Ok(_) => {},
        };
    });
}

pub fn get_unread_entries(conn: &Connection) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut stmt = conn
        .prepare("SELECT title, content, feed, guid, html_url FROM entries WHERE read = 0;")
        .unwrap();
    let result = stmt.query_map(params![], |row| {
        Ok(Entry {
            title: row.get(0).expect("Title not defined in the DB"),
            content: row.get(1).expect("Content not defined in the DB"),
            feed: row.get(2).expect("Feed not defined in the DB"),
            guid: row.get(3).expect("GUID not defined in the DB"),
            html_url: row.get(4).expect("HTML URL not defined in the DB."),
        })
    });

    match result {
        Ok(c) => Ok(c.filter(|e| e.is_ok()).map(|e| e.unwrap()).collect()),
        Err(e) => {
            eprintln!("Getting unread entries failed: {}", e);
            Err(Box::new(e))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_db_roundtrip() {
        let conn = get_connection(PathBuf::from("./test.db"));
        let entry = Entry {
            html_url: String::from("some.url"),
            title: String::from("A Title to an Entry"),
            content: String::from("Content for an entry!"),
            feed: String::from("A Cool Blog"),
            guid: String::from("some-cool-guid"),
        };
        insert_new_unread_entries(&conn, vec![entry.clone()]);
        let res = get_unread_entries(&conn).expect("Can't read unread entries");
        let res_entry = res.first().expect("No element found in unread entries");

        std::fs::remove_file(std::path::Path::new("./test.db")).expect("Can't delete test-db");

        assert_eq!(&entry, res_entry);
    }
}
