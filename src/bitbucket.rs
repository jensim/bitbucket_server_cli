use futures::stream::{self, StreamExt};
use generic_error::Result;
use reqwest::{header::ACCEPT, Client as ReqwestClient, RequestBuilder};
use serde::de::DeserializeOwned;

use crate::types::{self, AllProjects, BitBucketOpts, ProjDesc, Repo};

type BitbucketResult<T> = std::result::Result<T, BitbucketError>;

struct BitbucketError {
    msg: String,
    cause: String,
}

#[derive(Clone)]
pub struct Bitbucket {
    pub opts: BitBucketOpts,
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
        let fetch_result: Vec<BitbucketResult<Vec<Repo>>> =
            stream::iter(all_projects.iter().map(|project| {
                let opts = self.opts.clone();
                let key = project.key.clone();
                async move { fetch_one(key, opts).await }
            }))
            .buffer_unordered(self.opts.concurrency)
            .collect::<Vec<BitbucketResult<Vec<Repo>>>>()
            .await;

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
    let url = fetch_all_projects_url(&host);

    let response: reqwest::Result<reqwest::Response> = bake_client(url, &opts).send().await;
    Ok(extract_body::<AllProjects>(response, "projects".to_owned())
        .await?
        .values)
}

fn fetch_all_projects_url(host: &str) -> String {
    format!("{}/rest/api/1.0/projects?limit=100000", host)
}

async fn extract_body<T>(
    response: reqwest::Result<reqwest::Response>,
    naming: String,
) -> BitbucketResult<T>
where
    T: DeserializeOwned,
{
    match response {
        Ok(response) if response.status().is_success() => match response.json::<T>().await {
            Ok(all_projects) => Ok(all_projects),
            Err(e) => Err(BitbucketError {
                msg: format!(
                    "Failed fetching {} from bitbucket, bad json format.",
                    naming
                ),
                cause: format!("{:?}", e),
            }),
        },
        Ok(response) => Err(BitbucketError {
            msg: format!(
                "Failed fetching {} from bitbucket, status code: {}.",
                naming,
                response.status()
            ),
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
    let url = fetch_one_url(&opts.server.clone().unwrap().to_lowercase(), &project_key);
    let result: reqwest::Result<reqwest::Response> = bake_client(url, &opts).send().await;
    Ok(
        extract_body::<types::Projects>(result, format!("project {}", project_key))
            .await?
            .get_clone_links(&opts),
    )
}

fn fetch_one_url(host: &str, project_key: &str) -> String {
    format!(
        "{host}/rest/api/latest/projects/{key}/repos?limit=5000",
        host = host,
        key = project_key
    )
}

fn bake_client(url: String, opts: &BitBucketOpts) -> RequestBuilder {
    let builder: RequestBuilder = ReqwestClient::new()
        .get(url.trim())
        .header(ACCEPT, "application/json");
    match (&opts.username, &opts.password) {
        (Some(u), Some(p)) => builder.basic_auth(u.clone(), Some(p.clone())),
        _ => builder,
    }
}

#[cfg(test)]
mod tests {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    use crate::types::CloneType;

    use super::*;

    fn random_string(len: usize) -> String {
        thread_rng().sample_iter(&Alphanumeric).take(len).collect()
    }

    fn basic_opts() -> BitBucketOpts {
        BitBucketOpts {
            username: None,
            password: None,
            password_from_env: false,
            concurrency: 1,
            verbose: true,
            server: Some(format!(
                "http://{host}.p2/{path}",
                host = format!("{}", random_string(12)),
                path = format!("{}", random_string(12))
            )),
            clone_type: CloneType::HTTP,
        }
    }

    #[tokio::test]
    async fn fetch_one_bad_host_is_dns_error() {
        // given
        let project_key = "KEY".to_owned();
        let bit_bucket_opts = basic_opts();

        // when
        let result = fetch_one(project_key.clone(), bit_bucket_opts).await;

        // then
        match result {
            Ok(_) => assert!(false, "This request was expected to fail."),
            Err(e) => {
                assert_eq!(
                    e.msg,
                    format!("Failed fetching project {} from bitbucket.", &project_key),
                    "Unexpected error message. Was '{}'",
                    e.msg
                );
                assert!(
                    e.cause.contains("ConnectError(\"dns error\""),
                    format!("Was {}", e.cause)
                );
            }
        }
    }

    #[tokio::test]
    async fn fetch_one_bad_url_path_is_404() {
        // given
        let project_key = "KEY".to_owned();
        let mut bit_bucket_opts = basic_opts();
        bit_bucket_opts.server = Some("http://bitbucket.com/This_Will_Never_Work".to_owned());

        // when
        let result = fetch_one(project_key, bit_bucket_opts).await;

        // then
        match result {
            Ok(_) => assert!(false, "This request was expected to fail."),
            Err(e) => assert!(
                e.msg.contains(format!("status code: {}", 404).as_str()),
                "Response code should be 404."
            ),
        }
    }
}
