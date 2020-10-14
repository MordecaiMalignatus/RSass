use rusqlite::{params, Connection, Error::SqliteFailure};

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
  guid blob unique,
  html_url text
);
"#,
        params![],
    )
}

/// Mark an entry as read. This sets the `read` field in the SQLite database,
/// meaning it will not be selected by any future calls to `get_unread_entries`.
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

/// Inserts new entries into the database, deduplicating on the `guid` of each entry.
pub fn insert_new_entries(conn: &Connection, items: Vec<Entry>) -> () {
    let mut stmt = conn
        .prepare(
            "insert into entries(title, content, read, feed, guid, html_url) \
                  values (?, ?, 0, ?, ?, ?)",
        )
        .expect("Can't prepare statement for inserting unread entries.");

    items.into_iter().for_each(|entry| {
        match stmt.execute(params![
            entry.title,
            entry.content,
            entry.feed,
            entry.guid,
            entry.html_url,
        ]) {
            Err(e) => {
                match e {
                    SqliteFailure(e, Some(description)) => {
                        // Deduplication happens by running into the UNIQUE
                        // constraint, so we filter this out here by explicit
                        // check.
                        if !(description == "UNIQUE constraint failed: entries.guid") {
                            eprintln!(
                                "Coulnd't insert entry into DB: SqliteFailure: {}\n{:#?}",
                                e, entry
                            )
                        }
                    }
                    _ => eprintln!("Coulnd't insert entry into DB: {}\n{:#?}", e, entry),
                }
            }
            Ok(_) => {}
        };
    });
}

/// Select unread entries from the database. This assumes all entries to be
/// well-formed, and will panic if some of the are malformed in the DB.
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
    use std::{fs::remove_file, path::Path};

    use super::*;

    #[test]
    fn test_db_roundtrip() {
        let conn = get_connection(PathBuf::from("./test_roundtrip.db"));
        let entry = Entry {
            html_url: String::from("some.url"),
            title: String::from("A Title to an Entry"),
            content: String::from("Content for an entry!"),
            feed: String::from("A Cool Blog"),
            guid: String::from("some-cool-guid"),
        };
        insert_new_entries(&conn, vec![entry.clone()]);
        let res = get_unread_entries(&conn).expect("Can't read unread entries");
        let res_entry = res.first().expect("No element found in unread entries");

        remove_file(Path::new("./test_roundtrip.db")).expect("Can't delete test-db");
        assert_eq!(&entry, res_entry);
    }

    #[test]
    fn test_read_unread() {
        let conn = get_connection(PathBuf::from("./test_unread.db"));
        let entry = Entry {
            html_url: String::from("some.url"),
            title: String::from("A Title to an Entry"),
            content: String::from("Content for an entry!"),
            feed: String::from("A Cool Blog"),
            guid: String::from("some-cool-guid"),
        };
        insert_new_entries(&conn, vec![entry.clone()]);
        mark_entry_as_read(&conn, &entry.guid).expect("Marking entry as read failed");
        let res: Vec<Entry> = get_unread_entries(&conn).expect("Can't retrieve unread stories");

        remove_file(Path::new("./test_unread.db")).expect("Can't delete test-db");
        assert!(res.is_empty());
    }

    #[test]
    fn test_reinsert() {
        let conn = get_connection(PathBuf::from("./test_reinsert.db"));
        let entry = Entry {
            html_url: String::from("some.url"),
            title: String::from("A Title to an Entry"),
            content: String::from("Content for an entry!"),
            feed: String::from("A Cool Blog"),
            guid: String::from("some-cool-guid"),
        };
        insert_new_entries(&conn, vec![entry.clone()]);
        insert_new_entries(&conn, vec![entry.clone()]);
        let res: Vec<Entry> = get_unread_entries(&conn).expect("Can't retrieve unread stories");

        remove_file(Path::new("./test_reinsert.db")).expect("Can't delete test-db");
        assert_eq!(res.len(), 1);
        assert_eq!(res.first().unwrap(), &entry)
    }
}
