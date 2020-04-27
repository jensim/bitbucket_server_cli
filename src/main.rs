#![forbid(unsafe_code)]
extern crate reqwest;
#[macro_use]
extern crate serde;

use structopt::StructOpt;

use crate::{
    cloner::Cloner,
    types::Opts,
};

mod types;
mod cloner;
mod prompts;
mod bitbucket;
mod git;
mod input;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::from_args();
    match Cloner::new(opts).git_clone().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e.msg);
            std::process::exit(1);
        }
    }
}
