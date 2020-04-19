mod interface;
mod rss;
mod utils;
use std::path::Path;

fn main() {
    println!("Hello, world!");
    dbg!(rss::import_opml(&Path::new(
        "/Users/az/Downloads/feedly-7df722b7-feee-42df-b829-a8741ac1df35-2020-04-15.opml"
    )));
}
