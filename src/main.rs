#![forbid(unsafe_code)]
extern crate reqwest;
#[macro_use]
extern crate serde;

use generic_error::Result;
use reqwest::header::ACCEPT;

use types::Opts;
use types::Repo;
use reqwest::RequestBuilder;

mod types;
mod git;
mod input;
mod prompts;

fn main() {
    let opts: Opts = input::opts();
    download_project(&opts);
}

fn download_project(opts: &Opts) {
    let repos:Result<Vec<Repo>>;
    match opts.bit_bucket_project_all {
        true => {
            repos = fetch_all(&opts);
        },
        false => {
            let key = opts.bit_bucket_project_key.as_ref().unwrap();
            repos = fetch_one(key, &opts);
        },
    };
    match repos {
        Ok(l) => {
            git::git_going(&opts, l);
        }
        Err(_) => {
            eprintln!("Failed loading repository list from bitbucket");
            std::process::exit(1);
        }
    }
}

fn fetch_all(opts: &Opts) -> Result<Vec<Repo>> {
    let host = opts.bit_bucket_server.clone();
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
                      host = opts.bit_bucket_server.to_lowercase(),
                      key = project_key);
    let builder = bake_client(url, &opts);
    let projects: types::Projects = builder.send()?.json()?;
    return Ok(projects.get_clone_links());
}

fn bake_client(url: String, opts: &Opts) -> RequestBuilder {
    let client = reqwest::Client::new();
    let mut builder = client.get(url.trim());
    builder = builder.header(ACCEPT, "application/json");
    return match (&opts.bit_bucket_username, opts.bit_bucket_password.as_ref()) {
        (u, Some(p)) => builder.basic_auth(u.clone(), Some(p.clone())),
        _ => builder,
    }
}
