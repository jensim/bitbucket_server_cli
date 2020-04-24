#![forbid(unsafe_code)]
extern crate reqwest;
#[macro_use]
extern crate serde;

use std::pin::Pin;

use futures::{
    Future,
    future::FutureExt,
    future::join_all,
};
use generic_error::Result;
use reqwest::{
    Client as ReqwestClient,
    header::ACCEPT,
    RequestBuilder,
};
use structopt::StructOpt;

use crate::{
    input::select_projects,
    types::{
        AllProjects,
        Opts,
        ProjDesc,
        Repo,
    },
};

mod types;
mod git;
mod input;
mod prompts;

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
    let mut repos: Vec<Repo> = match fetch_all(opts.clone()).await {
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
    git::git_going(&opts, &repos);
    Ok(())
}

type MyDynFuture = Pin<Box<dyn Future<Output=Result<Vec<Repo>>>>>;

async fn fetch_all(opts: Opts) -> Result<Vec<Repo>> {
    let mut all: Vec<Repo> = Vec::new();
    let all_projects: Vec<ProjDesc> = match fetch_all_projects(&opts).await {
        Ok(v) => v,
        Err(e) => panic!("Failed fetching projects from bitbucket. {}", e.msg),
    };
    let i: Vec<MyDynFuture> = all_projects.iter().map(|p: &ProjDesc| -> MyDynFuture { fetch_one(p.key.clone(), opts.clone()).boxed() }).collect();
    let all_repo_requests: Vec<Result<Vec<Repo>>> = join_all(i).await;
    for response in all_repo_requests {
        match response {
            Ok(repos) => {
                for repo in repos {
                    all.push(repo);
                }
            },
            Err(_e) => {}
        };
    }
    Ok(all)
}

async fn fetch_all_projects(opts: &Opts) -> Result<Vec<ProjDesc>> {
    let host = opts.bit_bucket_server.clone().unwrap();
    let url = format!("{}/rest/api/1.0/projects?limit=100000", host);
    Ok(bake_client(url, opts).send().await?.json::<AllProjects>().await?.values)
}

async fn fetch_one(project_key: String, opts: Opts) -> Result<Vec<Repo>> {
    let url = format!("{host}/rest/api/latest/projects/{key}/repos?limit=5000",
                      host = opts.bit_bucket_server.clone().unwrap().to_lowercase(),
                      key = project_key);
    let projects: types::Projects = bake_client(url, &opts).send().await?
        .json::<types::Projects>().await?;
    return Ok(projects.get_clone_links());
}

fn bake_client(url: String, opts: &Opts) -> RequestBuilder {
    let builder: RequestBuilder = ReqwestClient::new().get(url.trim())
        .header(ACCEPT, "application/json");
    return match (&opts.bit_bucket_username, &opts.bit_bucket_password) {
        (Some(u), Some(p)) => builder.basic_auth(u.clone(), Some(p.clone())),
        _ => builder,
    };
}
