extern crate virustotal3;
use virustotal3::*;

use regex::Regex;
use log::{debug,trace,error};

// 1- Function to parse URL to get url,id
// 2- Function to get comments
// 3- Function to post comment

/// Function to parse Virustotal url
pub fn parse_virustotal_url(full_url: &String) -> (String,VtType,String) {
    // https://www.virustotal.com/gui/file/99ff0b679081cdca00eb27c5be5fd9428f1a7cf781cc438b937cf8baf8551c4d
    let re = Regex::new(r"^(https://[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}/)").unwrap();
    let caps = re.captures(&full_url).unwrap();
    let url = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("URL: {}",&url));

    let re = Regex::new(r"^https://[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}/gui/([a-z-]{2,})/[a-zA-Z0-9.()]{3,}").unwrap();
    let caps = re.captures(&full_url).unwrap();
    let vtype = caps.get(1).map_or("", |m| m.as_str());
    let mut vt_type = VtType::File;
    match vtype {
        "files" => { vt_type = VtType::File; }
        "domains" => { vt_type = VtType::Domain; }
        "urls" => { vt_type = VtType::Url; }
        "ip-address" => { vt_type = VtType::Url; }
        _ => { }
    }
    debug!("{}",format!("TYPE: {}",&vtype));

    let re = Regex::new(r"^https://[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}/gui/file/([a-zA-Z0-9_]{32,})").unwrap();
    let caps = re.captures(&full_url).unwrap();
    let id = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("ID: {}",&id));

    return (url,vt_type,id)
}


/// GET comment in one resource from the ID
pub async fn virustotal_get_comments(
    access_token: &str,
    resource_id: &str,
    vtype: &VtType,
    filter: String,
) -> (Vec<String>,Vec<String>,Vec<String>) {

    let vt = VtClient::new(access_token);

    // Result
    let mut result_spoiler: Vec<String> = Vec::new();
    let mut result_content: Vec<String> = Vec::new();
    let mut post_id: Vec<String> = Vec::new();

    match vt.get_comment(resource_id, vtype).await {
        Ok(result) => {
            if result.data.len() >= 1 {
                // Comment by comment
                for c in result.data {

                    if c.attributes.text.contains(&filter) {
                        let split = c.attributes.text.split("\n");
                        let collection = split.collect::<Vec<&str>>();
                        let re = Regex::new(r"[0-9a-f]+").unwrap();
                        for hexa in re.captures_iter(&collection[1])
                        {
                            trace!("Getting comment in resource: {}",resource_id);
                            result_spoiler.push(collection[0].to_owned());
                            result_content.push(hexa[0].to_owned().to_string());
                            post_id.push(c.id.to_string());
                        }
                    }
                }
            }
            else {
                error!("Cant get comments in this resource..");
                panic!("Session killed?")
            }
        },
        Err(err) => {
            error!("Cant get comments in this resource: {err}")
        }
    }
    return (result_spoiler,result_content,post_id)
}

/// POST private comment
pub async fn virustotal_post_comment(
    access_token: &str,
    resource_id: &str,
    vtype: &VtType,
    datas: &str,
) {
    let vt = VtClient::new(access_token);
    match vt.put_comment(resource_id, datas, vtype).await {
        Ok(result) => {
            trace!("Post answer {:?}", result.data);
        },
        Err(err) => {
            error!("Cant post comment in this resource: {err}")
        }
    }
}