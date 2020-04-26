use futures::stream::{self, StreamExt};
use generic_error::Result;
use reqwest::{
    Client as ReqwestClient,
    header::ACCEPT,
    RequestBuilder,
};
use serde::de::DeserializeOwned;

use crate::types::{AllProjects, BitBucketOpts, ProjDesc, Repo};
use crate::types;

type BitbucketResult<T> = std::result::Result<T, BitbucketError>;

struct BitbucketError {
    msg: String,
    cause: String,
}

#[derive(Clone)]
pub struct Bitbucket {
    pub opts: BitBucketOpts
}

impl Bitbucket {
    pub async fn fetch_all(self) -> Result<Vec<Repo>> {
        let all_projects: Vec<ProjDesc> = match fetch_all_projects(self.opts.clone()).await {
            Ok(v) => v,
            Err(e) => {
                if self.opts.verbose {
                    eprintln!("{}\nCause: {}", e.msg, e.cause);
                } else {
                    eprintln!("{}", e.msg);
                }
                std::process::exit(1);
            }
        };
        let fetch_result: Vec<BitbucketResult<Vec<Repo>>> = stream::iter(
            all_projects.iter().map(|project| {
                let opts = self.opts.clone();
                let key = project.key.clone();
                async move {
                    fetch_one(key, opts).await
                }
            })
        ).buffer_unordered(self.opts.concurrency).collect::<Vec<BitbucketResult<Vec<Repo>>>>().await;
        let mut all: Vec<Repo> = Vec::new();
        for response in fetch_result {
            match response {
                Ok(repos) => {
                    for repo in repos {
                        all.push(repo);
                    }
                }
                Err(e) => {
                    if self.opts.verbose {
                        eprintln!("{} Cause: {}", e.msg, e.cause);
                    }
                }
            };
        }
        Ok(all)
    }
}

async fn fetch_all_projects(opts: BitBucketOpts) -> BitbucketResult<Vec<ProjDesc>> {
    let host = opts.server.clone().unwrap();
    let url = format!("{}/rest/api/1.0/projects?limit=100000", host);

    let response: reqwest::Result<reqwest::Response> = bake_client(url, opts).send().await;
    Ok(extract_body::<AllProjects>(response, "projects".to_owned()).await?.values)
}

async fn extract_body<T>(response: reqwest::Result<reqwest::Response>, naming: String) -> BitbucketResult<T> where T: DeserializeOwned {
    match response {
        Ok(response) if response.status().is_success() => match response.json::<T>().await {
            Ok(all_projects) => Ok(all_projects),
            Err(e) => Err(BitbucketError {
                msg: format!("Failed fetching {} from bitbucket, bad json format.", naming),
                cause: format!("{:?}", e),
            }),
        },
        Ok(response) => Err(BitbucketError {
            msg: format!("Failed fetching {} from bitbucket, status code: {}.", naming, response.status()),
            cause: match response.text().await {
                Ok(t) => format!("Body: '{}'", t),
                Err(e) => format!("Body: '#unable_to_parse', Err: {:?}", e),
            },
        }),
        Err(e) => Err(BitbucketError {
            msg: format!("Failed fetching {} from bitbucket.", naming),
            cause: format!("{:?}", e),
        }),
    }
}

async fn fetch_one(project_key: String, opts: BitBucketOpts) -> BitbucketResult<Vec<Repo>> {
    let url = format!("{host}/rest/api/latest/projects/{key}/repos?limit=5000",
                      host = opts.server.clone().unwrap().to_lowercase(),
                      key = project_key.clone());
    let result: reqwest::Result<reqwest::Response> = bake_client(url, opts).send().await;
    Ok(extract_body::<types::Projects>(result, format!("project {}", project_key)).await?.get_clone_links())
}

fn bake_client(url: String, opts: BitBucketOpts) -> RequestBuilder {
    let builder: RequestBuilder = ReqwestClient::new().get(url.trim())
        .header(ACCEPT, "application/json");
    return match (&opts.username, &opts.password) {
        (Some(u), Some(p)) => builder.basic_auth(u.clone(), Some(p.clone())),
        _ => builder,
    };
}
