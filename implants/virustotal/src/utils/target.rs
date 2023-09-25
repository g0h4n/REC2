use log::trace;

/// Function to get target information
/// <https://docs.rs/whoami/latest/whoami/>
pub fn get_target_info() -> String {
    trace!("Username: {}",whoami::username());
    trace!("Hostname: {}",whoami::hostname());
    trace!("OS: {}", whoami::distro());
    format!("{}:-:{}:-:{}",whoami::hostname(),whoami::username(),whoami::distro())
}

/// Function to get first 8 chars from uniq md5 target hostname
pub fn get_target_hash() -> String {
    let s = format!("{}:{}:{}",whoami::hostname(),whoami::username(),whoami::distro());
    let digest = md5::compute(s.as_bytes());
    let hash = format!("{:x}",digest);
    trace!("Uniq hash for this target: {}", hash);
    return hash[0..8].to_owned()
}