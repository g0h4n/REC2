//! Parsing arguments
use clap::{Arg,ArgAction,Command};

#[derive(Debug)]
pub struct Options {
    pub verbose: log::LevelFilter,
}

fn cli() -> Command {
        Command::new("rec2")
            .about("REC2 implant for VirusTotal")
            .arg(Arg::new("v")
                .short('v')
                .help("Set the level of verbosity")
                .action(ArgAction::Count),
            )
}

/// Function to extract arguments
pub fn extract_args() -> Options {
        let matches = cli().get_matches();
        let v = match matches.get_count("v") {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };
        Options {
            verbose: v,
        }
}