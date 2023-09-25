use regex::Regex;
use log::debug;

/// Function to parse header
pub fn parse_spoiler(spoiler: &String) -> (String, String, String) {
    let re = Regex::new(r"^([A-Z0-9]{4,}):[a-zA-Z0-9]{8}:[a-zA-Z0-9]{6}:").unwrap();
    let caps = re.captures(&spoiler).unwrap();
    let msg_type = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("{:<15}: {}","MSG_TYPE:",&msg_type));

    let re = Regex::new(r"^[A-Z0-9]{4,}:([a-zA-Z0-9]{8}):").unwrap();
    let caps = re.captures(&spoiler).unwrap();
    let session_hash = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("{:<15}: {}","SESSION_HASH",&session_hash));

    let re = Regex::new(r"^[A-Z0-9]{4,}:[a-zA-Z0-9]{8}:([a-zA-Z0-9]{6}):").unwrap();
    let caps = re.captures(&spoiler).unwrap();
    let job_hash = caps.get(1).map_or("", |m| m.as_str()).to_string();
    debug!("{}",format!("{:<15}: {}","JOB_HASH",&job_hash));

    return (msg_type, session_hash, job_hash)
}