use std::error::Error;

use anstyle::AnsiColor;
use clap::Parser;

use jukctl::cli::Args;
use jukctl::logger::Logger;

fn main() {
    let args = Args::parse();
    Logger::init(args.verbosity.filter());
    if let Err(e) = jukctl::run(args) {
        eprintln!(
            "\n{0}error{0:#}: {1}",
            AnsiColor::BrightRed.on_default().bold(),
            e
        );

        let mut source = e.source();
        if source.is_some() {
            eprintln!("\nCaused by:");
            let mut i = 0;

            while let Some(err) = source {
                eprintln!("{}: {}", i, err);
                source = err.source();
                i += 1;
            }
        }
        std::process::exit(-1);
    }
}
