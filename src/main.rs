mod utils;
mod rss;
mod display;
mod cli;

use cli::run;

use anyhow::Result;
use colored::*;

#[allow(dead_code)]
#[allow(unused_imports)]
use utils::format_date;



#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run().await {
        eprintln!("{} {:#}", "ERROR:".red().bold(), e);
        std::process::exit(1);
    }
    Ok(())
}