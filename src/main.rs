mod interface;
mod rss;
mod utils;

fn main() {
    let channels = rss::load_feeds(&utils::read_feeds());
    dbg!(channels.get(1).unwrap().items());
}
