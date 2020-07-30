mod interface;
mod rss;
mod utils;

use clap::{App, Arg, SubCommand};

fn main() {
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
        .get_matches();

    match args.subcommand_name() {
        Some("import") => {},
        Some(_) => panic!("Unknown subcommand"),
        None => {
            interface::make_window()
        },
    }
}
