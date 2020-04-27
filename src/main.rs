#![forbid(unsafe_code)]
extern crate reqwest;
#[macro_use]
extern crate serde;

use structopt::StructOpt;

use crate::cloner::Cloner;
use crate::types::Opts;

mod types;
mod git;
mod input;
mod prompts;
mod bitbucket;
mod cloner;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::from_args();
    Cloner::new(opts).clone().await.unwrap();
    Ok(())
}
