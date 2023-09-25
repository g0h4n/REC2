pub mod modules;
pub mod utils;

pub mod args;
use args::*;

pub mod c2;
use c2::shell::Shell;

use env_logger::Builder;
use rustyline::Result;

#[tokio::main]
/// Main function to start C2 terminal
async fn main() -> Result<()> {
    // Get args
    let common_args = extract_args();
    // Build logger
    Builder::new()
        .filter(Some("server"), common_args.verbose)
        .filter_level(log::LevelFilter::Error)
        .init();
    // Get shell prompt
    let _handle = Shell::new(common_args).await.run().await;
    Ok(())
}