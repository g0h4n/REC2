use std::{thread, time};
use random_string::generate;
use log::{info,trace};

///Display cmd output
pub fn display_vecu8(output: &Vec<u8>) -> String {
    return String::from_utf8_lossy(output).to_string();
}

/// Function sleep
pub fn sleep(timer: u64) {
    info!("[?] Waiting {}s",&timer);
    thread::sleep(time::Duration::from_secs(timer));
}

/// Function to generate random string
pub fn random_string(len: usize) -> String {
    let charset = "1234567890abcdefghijklmnopqrstuvwxyz";
    let r = generate(len, charset);
    trace!("Random ID: {}",&r);
    return r
}