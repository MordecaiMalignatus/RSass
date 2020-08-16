mod db;
mod interface;
mod rss;
mod utils;

use clap::{App, Arg, SubCommand};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new("rsass")
        .version("0.1")
        .author("Mordecai Malignatus")
        .about("File-based RSS reader")
        .subcommand(
            SubCommand::with_name("import")
                .about("Imports from OPML")
                .arg(
                    Arg::with_name("FILE")
                        .required(true)
                        .index(1)
                        .help("OPML file to import"),
                ),
        )
        .subcommand(
            SubCommand::with_name("fetch").about("Fetch and store feeds without launching viewer"),
        )
        .get_matches();

    match args.subcommand() {
        ("import", Some(import_matches)) => {
            let path = import_matches.value_of("FILE").expect("FILE is required");
            let path = PathBuf::from(path);
            rss::import_opml(&path)
        }
        (unknown, Some(_)) => panic!("Unknown subcommand: {}", unknown),
        _ => {
            interface::make_window();
            Ok(())
        }
    }
}
