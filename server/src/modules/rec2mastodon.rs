use megalodon::{
    generator,
    SNS,
    megalodon::GetStatusContextInputOptions,
    megalodon::PostStatusInputOptions,
    entities::status::StatusVisibility,
};
use regex::Regex;
use log::{debug,trace};
use colored::*;

// 1- Function to parse URL to get url,username,topic_id
// 2- Function to get topic comments
// 3- Function to post comment in topic_id
// 4- Function to remove comment status

/// Function to parse Mastodon url
pub fn parse_mastodon_url(full_url: &String) -> (String, String, String) {
    let re = Regex::new(r"^(https://[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}/)").unwrap();
    let caps = re.captures(&full_url).unwrap();
    let url = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("{:<10}: {}","URL".cyan().bold(),&url));

    let re = Regex::new(r"^https://[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}/@([a-zA-Z0-9_]{10,})/").unwrap();
    let caps = re.captures(&full_url).unwrap();
    let username = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("{:<10}: {}","USERNAME".cyan().bold(),&username));

    let re = Regex::new(r"([0-9]{10,})").unwrap();
    let caps = re.captures(&full_url).unwrap();
    let topic = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("{:<10}: {}","TOPIC ID".cyan().bold(),&topic));

    return (url, username, topic)
}

/// GET comment in one topic ID
/// <https://docs.rs/megalodon/0.3.7/megalodon/megalodon/trait.Megalodon.html#tymethod.get_status_context>
pub async fn mastodon_get_topic_comments(
    url: &String,
    access_token: String,
    topic_id: String,
    filter: String,
) -> (Vec<String>, Vec<String>, Option<String>, Vec<String>) {
    let client = generator(
        SNS::Mastodon, 
        url.to_string(), 
        Some(access_token), 
        None);
    let status = client.get_status_context(
        topic_id, 
        Some(&GetStatusContextInputOptions {
            limit: Some(9999),
            max_id: Some("".to_string()),
            since_id: Some("".to_string()),
        }
    )).await.expect("[-] Could not contact URL, please check it..\n");
    let status = status.json().descendants;
    //debug!("{:?}",&status);
    let mut in_reply_to_id = Some("null".to_string());
    let mut post_id: Vec<String> = Vec::new();
    // Result
    let mut result_spoiler: Vec<String> = Vec::new();
    let mut result_content: Vec<String> = Vec::new();

    // Comment by comment
    for s in status {
        //info!("{:?}",s.content);
        // If spoiler_text empty command to run and not an answer
        if s.spoiler_text.contains(&filter) {
            // If is hexa so it's encoded datas
            let re = Regex::new(r"[0-9a-f]+").unwrap();
            for hexa in re.captures_iter(&s.content)
            {
                trace!("Getting comment in topic: {:?}",hexa[0].to_owned().to_string());
                in_reply_to_id = s.to_owned().in_reply_to_id;
                post_id.push(s.to_owned().id);
                result_spoiler.push(s.spoiler_text.to_owned().to_string());
                result_content.push(hexa[0].to_owned().to_string());
            }
        }
    }
    return (result_spoiler,result_content,in_reply_to_id,post_id)
}

/// POST private answer
/// <https://docs.rs/megalodon/0.3.7/megalodon/megalodon/trait.Megalodon.html#tymethod.post_status>
/// <https://github.com/h3poteto/megalodon-rs/blob/master/examples/mastodon_post_with_media.rs>
pub async fn mastodon_post_topic_comment(
    url: &String,
    access_token: String,
    in_reply_to_id: Option<String>,
    datas: String,
    sploiler_text: String,
) {
    let client = generator(
        SNS::Mastodon, 
        url.to_string(), 
        Some(access_token), 
        None);
    let status = client.post_status(
        datas, 
        Some(&PostStatusInputOptions {
            media_ids: Some(Vec::new()),
            in_reply_to_id: in_reply_to_id,
            sensitive: Some(true),
            visibility: Some(StatusVisibility::Private),
            language: Some("en".to_string()),
            spoiler_text: Some(sploiler_text),
            ..Default::default()
        }),
    ).await.expect("[-] Could not contact URL, please check it..\n");
    let status = status.json();
    debug!("Post answer ok");
    trace!("Post answer {:?}", status);
}

/// Function to REMOVE comments status
/// <https://docs.rs/megalodon/latest/megalodon/megalodon/trait.Megalodon.html#tymethod.delete_status>
pub async fn remove_mastodon_comment(
    url: &String,
    access_token: String,
    id: String,
) {
    let client = generator(
        SNS::Mastodon, 
        url.to_string(), 
        Some(access_token), 
        None);
    let _status = client.delete_status(
        id.to_owned(),
    ).await;
    debug!("Delete comment ID:{} from Mastodon topic",id);
}