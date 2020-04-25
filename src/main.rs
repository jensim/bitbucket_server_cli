#![forbid(unsafe_code)]
extern crate reqwest;
#[macro_use]
extern crate serde;

use structopt::StructOpt;

use crate::{
    bitbucket::Bitbucket,
    git::Git,
    input::select_projects,
    types::{
        Opts,
        Repo,
    },
};

mod types;
mod git;
mod input;
mod prompts;
mod bitbucket;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut opts: Opts = Opts::from_args();

    if opts.interactive {
        opts = input::opts(&opts);
    } else if opts.bit_bucket_server.is_none() {
        println!("server is required");
        std::process::exit(1);
    } else if !opts.bit_bucket_project_all && opts.bit_bucket_project_keys.is_empty() {
        println!("project selection is required (all or keys)");
        std::process::exit(1);
    }
    let bb = Bitbucket { opts: opts.clone() };
    let mut repos: Vec<Repo> = match bb.fetch_all().await {
        Ok(r) => r,
        _ => {
            println!("Failed fetching repos from bitbucket");
            std::process::exit(1);
        }
    };
    if opts.interactive && !opts.bit_bucket_project_all && opts.bit_bucket_project_keys.is_empty() {
        opts.bit_bucket_project_keys = select_projects(&repos);
    }

    if !opts.bit_bucket_project_all && !opts.bit_bucket_project_keys.is_empty() {
        let mut tmp_vec = Vec::new();
        for r in repos {
            if opts.bit_bucket_project_keys.contains(&r.project_key) {
                tmp_vec.push(r);
            }
        }
        repos = tmp_vec;
    }
    Git { opts, repos }.git_going().await;
    Ok(())
}
