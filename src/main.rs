#![forbid(unsafe_code)]
extern crate reqwest;
#[macro_use]
extern crate serde;

use generic_error::Result;
use reqwest::header::ACCEPT;
use reqwest::RequestBuilder;
use structopt::StructOpt;

use types::Opts;
use types::Repo;

use crate::input::select_projects;

mod types;
mod git;
mod input;
mod prompts;

fn main() {
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
    let mut repos: Vec<Repo> = match fetch_all(&opts) {
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
    git::git_going(&opts, repos);
}

fn fetch_all(opts: &Opts) -> Result<Vec<Repo>> {
    let host = opts.bit_bucket_server.clone().unwrap();
    let url = format!("{}/rest/api/1.0/projects?limit=100000", host);
    let builder = bake_client(url, &opts);
    let projects: types::AllProjects = builder.send()?
        .json()?;
    let mut all: Vec<Repo> = Vec::new();
    for value in projects.values {
        match fetch_one(&value.key, &opts) {
            Ok(project_repos) => for repo in project_repos {
                all.push(repo);
            }
            Err(_) => {
                eprintln!("Failed loading repository list from bitbucket project with key {}", value.key);
            }
        }
    }
    return Ok(all);
}

fn fetch_one(project_key: &String, opts: &Opts) -> Result<Vec<Repo>> {
    let url = format!("{host}/rest/api/latest/projects/{key}/repos?limit=5000",
                      host = opts.bit_bucket_server.clone().unwrap().to_lowercase(),
                      key = project_key);
    let builder = bake_client(url, &opts);
    let projects: types::Projects = builder.send()?.json()?;
    return Ok(projects.get_clone_links());
}

fn bake_client(url: String, opts: &Opts) -> RequestBuilder {
    let client = reqwest::Client::new();
    let mut builder = client.get(url.trim());
    builder = builder.header(ACCEPT, "application/json");
    return match (&opts.bit_bucket_username, &opts.bit_bucket_password) {
        (Some(u), Some(p)) => builder.basic_auth(u.clone(), Some(p.clone())),
        _ => builder,
    }
}
