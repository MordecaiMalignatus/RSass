use crate::rss::Entry;
use crate::utils;
use rusqlite::{params, Connection};

pub fn get_connection() -> Connection {
    let path = utils::db_path();
    match Connection::open(&path) {
        Ok(c) => {
            create_tables(&c);
            c
        }
        Err(e) => {
            panic!("Can't open DB file: {}", e);
        }
    }
}

fn create_tables(conn: &Connection) -> () {
    conn.execute(
        "
CREATE TABLE IF NOT EXISTS entries (
  title text,
  content text,
  read integer,
  guid blob
);
",
        params![],
    );
}

pub fn insert_unread_entries(conn: &Connection, items: Vec<Entry>) -> () {
    let mut stmt = conn
        .prepare("insert into entries(title, content, read, guid) values (?, ?, 0, ?)")
        .expect("Can't prepare statement for inserting unread entries.");

    items.into_iter().for_each(|entry| {
        stmt.execute(params![
            entry.title,
            entry.rss_entry.content(),
            entry.rss_entry.guid().unwrap().value()
        ]).map_err(|e|{eprintln!("Coulnd't insert entry into DB: {}\n{:#?}", e, entry)});
    });
}
