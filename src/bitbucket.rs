use futures::stream::{self, StreamExt};
use generic_error::Result;
use reqwest::{
    Client as ReqwestClient,
    header::ACCEPT,
    RequestBuilder,
};

use crate::types::{AllProjects, BitBucketOpts, ProjDesc, Repo};
use crate::types;

#[derive(Clone)]
pub struct Bitbucket {
    pub opts: BitBucketOpts
}

impl Bitbucket {
    pub async fn fetch_all(self) -> Result<Vec<Repo>> {
        let mut all: Vec<Repo> = Vec::new();
        let all_projects: Vec<ProjDesc> = match fetch_all_projects(self.opts.clone()).await {
            Ok(v) => v,
            Err(e) => panic!("Failed fetching projects from bitbucket. {}", e.msg),
        };
        let fetch_result: Vec<Result<Vec<Repo>>> = stream::iter(
            all_projects.iter().map(|project| {
                let opts = self.opts.clone();
                let key = project.key.clone();
                async move {
                    fetch_one(key, opts).await
                }
            })
        ).buffer_unordered(self.opts.concurrency).collect::<Vec<Result<Vec<Repo>>>>().await;
        for response in fetch_result {
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
}

async fn fetch_all_projects(opts: BitBucketOpts) -> Result<Vec<ProjDesc>> {
    let host = opts.server.clone().unwrap();
    let url = format!("{}/rest/api/1.0/projects?limit=100000", host);
    Ok(bake_client(url, opts).send().await?.json::<AllProjects>().await?.values)
}

async fn fetch_one(project_key: String, opts: BitBucketOpts) -> Result<Vec<Repo>> {
    let url = format!("{host}/rest/api/latest/projects/{key}/repos?limit=5000",
                      host = opts.server.clone().unwrap().to_lowercase(),
                      key = project_key);
    let projects: types::Projects = bake_client(url, opts).send().await?
        .json::<types::Projects>().await?;
    return Ok(projects.get_clone_links());
}

fn bake_client(url: String, opts: BitBucketOpts) -> RequestBuilder {
    let builder: RequestBuilder = ReqwestClient::new().get(url.trim())
        .header(ACCEPT, "application/json");
    return match (&opts.username, &opts.password) {
        (Some(u), Some(p)) => builder.basic_auth(u.clone(), Some(p.clone())),
        _ => builder,
    };
}
