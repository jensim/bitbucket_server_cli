use futures::stream::{self, StreamExt};
use generic_error::{GenericError, Result};
use reqwest::{header::ACCEPT, Client as ReqwestClient, RequestBuilder};
use serde::de::DeserializeOwned;

use crate::bitbucket::types::{
    AllProjects, AllUsersResult, ProjDesc, Projects, Repo, RepoUrlBuilder, UserResult,
};
use crate::types::BitBucketOpts;
use crate::util::bail;

pub type BitbucketResult<T> = std::result::Result<T, BitbucketError>;

pub struct BitbucketError {
    msg: String,
    cause: String,
}

#[derive(Clone)]
pub struct BitbucketWorker {
    opts: BitBucketOpts,
}

impl BitbucketWorker {
    pub fn new(opts: BitBucketOpts) -> BitbucketWorker {
        BitbucketWorker { opts }
    }

    pub async fn fetch_all_repos(&self) -> Result<Vec<Repo>> {
        let user_repos = self.fetch_all_user_repos().await;
        let project_repos = self.fetch_all_project_repos().await;
        match (user_repos, project_repos) {
            (Ok(mut u), Ok(mut p)) => {
                u.append(&mut p);
                Ok(u)
            }
            (Err(GenericError { msg }), Ok(p)) => {
                eprintln!("Failed loading user repos due to '{}'", msg);
                Ok(p)
            }
            (Ok(u), Err(GenericError { msg })) => {
                eprintln!("Failed loading project repos due to '{}'", msg);
                Ok(u)
            }
            (Err(user_e), Err(project_e)) => bail(&format!(
                "Failed loading user repos due to '{}'. Failed loading project repos due to '{}'",
                user_e.msg, project_e.msg
            )),
        }
    }

    pub async fn fetch_all_project_repos(&self) -> Result<Vec<Repo>> {
        match self.fetch_all_projects().await {
            Ok(all_projects) => Ok(self.fetch_all(all_projects).await?),
            Err(e) if self.opts.verbose => bail(&format!("{}\nCause: {}", e.msg, e.cause))?,
            Err(e) => bail(&e.msg)?,
        }
    }

    pub async fn fetch_all_user_repos(&self) -> Result<Vec<Repo>> {
        match self.fetch_all_users().await {
            Ok(all_users) => Ok(self.fetch_all(all_users).await?),
            Err(e) if self.opts.verbose => bail(&format!("{}\nCause: {}", e.msg, e.cause))?,
            Err(e) => bail(&e.msg)?,
        }
    }

    async fn fetch_all<T>(&self, all_projects: Vec<T>) -> Result<Vec<Repo>>
    where
        T: RepoUrlBuilder,
    {
        let keys: Vec<String> = self
            .opts
            .project_keys
            .iter()
            .map(|k| k.to_lowercase())
            .collect();
        let fetch_result: Vec<BitbucketResult<Vec<Repo>>> = stream::iter(
            all_projects
                .iter()
                .filter(|t| keys.is_empty() || keys.contains(&t.get_filter_key()))
                .map(|project| async move { self.fetch_one_project(project).await }),
        )
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

    async fn fetch_all_users(&self) -> BitbucketResult<Vec<UserResult>> {
        println!("Started fetching users..");
        let host = self.opts.server.clone().unwrap();
        let url = format!("{}/rest/api/1.0/users?limit=5000", host);

        let response: reqwest::Result<reqwest::Response> = self.bake_client(url).send().await;
        Ok(extract_body::<AllUsersResult>(response, "users")
            .await?
            .values)
    }

    async fn fetch_all_projects(&self) -> BitbucketResult<Vec<ProjDesc>> {
        println!("Started fetching projects..");
        let host = self.opts.server.clone().unwrap();
        let url = format!("{}/rest/api/1.0/projects?limit=100000", host);

        let response: reqwest::Result<reqwest::Response> = self.bake_client(url).send().await;
        Ok(extract_body::<AllProjects>(response, "projects")
            .await?
            .values)
    }

    async fn fetch_one_project<T>(&self, project: &T) -> BitbucketResult<Vec<Repo>>
    where
        T: RepoUrlBuilder,
    {
        let host = self.opts.server.clone().unwrap().to_lowercase();
        let url = project.get_repos_url(&host);
        let result: reqwest::Result<reqwest::Response> = self.bake_client(url).send().await;
        Ok(
            extract_body::<Projects>(result, &format!("project {:?}", project))
                .await?
                .get_clone_links(&self.opts),
        )
    }

    fn bake_client(&self, url: String) -> RequestBuilder {
        let builder: RequestBuilder = ReqwestClient::new()
            .get(url.trim())
            .header(ACCEPT, "application/json");
        match (&self.opts.username, &self.opts.password) {
            (Some(u), Some(p)) => builder.basic_auth(u.clone(), Some(p.clone())),
            _ => builder,
        }
    }
}

async fn extract_body<T>(
    response: reqwest::Result<reqwest::Response>,
    naming: &str,
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
            project_keys: vec!["key".to_owned()],
            all: false,
        }
    }

    #[tokio::test]
    async fn fetch_one_bad_host_is_dns_error() {
        // given
        let project: ProjDesc = ProjDesc {
            key: "KEY".to_owned(),
        };
        let bit_bucket_opts = basic_opts();
        let worker = BitbucketWorker::new(bit_bucket_opts);

        // when
        let result = worker.fetch_one_project(&project).await;

        // then
        match result {
            Ok(_) => assert!(false, "This request was expected to fail."),
            Err(e) => {
                assert_eq!(
                    e.msg,
                    format!("Failed fetching project {:?} from bitbucket.", &project),
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
        let project: ProjDesc = ProjDesc {
            key: "KEY".to_owned(),
        };
        let mut bit_bucket_opts = basic_opts();
        bit_bucket_opts.server = Some("http://bitbucket.com/This_Will_Never_Work".to_owned());
        let worker = BitbucketWorker::new(bit_bucket_opts);

        // when
        let result = worker.fetch_one_project(&project).await;

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
