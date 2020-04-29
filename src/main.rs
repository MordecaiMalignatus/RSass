mod interface;
mod rss;
mod utils;

fn main() {
    let channels = utils::read_feeds();
    dbg!(channels.len());
}
