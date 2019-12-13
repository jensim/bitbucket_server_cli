extern crate reqwest;
#[macro_use]
extern crate serde;

use structopt::StructOpt;

use generic_error::{Result, GenericError};
use reqwest::header::ACCEPT;
use types::Opts;
use types::Repo;
use std::fs::ReadDir;

mod types;
mod git;

fn main() {
    let opt = Opts::from_args();
    println!("Value for config: {:?}", opt);
    download_project(opt);
}

fn download_project(opts: Opts) {
    println!("Hello download_project");

    let url = opts.bit_bucket_url();
    match fetch(&url[..]) {
        Ok(l) => {
            println!("{:?}", l);
            git::git_going(l);
        },
        Err(e) => {
            eprintln!("Failed loading repository list from bitbucket");
            std::process::exit(1);
        },
    }
}


fn fetch(url: &str) -> Result<Vec<Repo>> {
    let client = reqwest::Client::new();
    let projects: types::Projects = client.get(url)
        .header(ACCEPT, "application/json")
        .send()?
        .json()?;
    return Ok(projects.get_clone_links());
}
