use std::borrow::BorrowMut;

use atomic_counter::{AtomicCounter, RelaxedCounter};
use futures::stream::{self, StreamExt};
#[allow(unused_imports)]
use futures::SinkExt as _;
use generic_error::{GenericError, Result};
use indicatif::ProgressStyle;
use reqwest::{header::ACCEPT, Client as ReqwestClient, RequestBuilder};
use serde::de::DeserializeOwned;

use crate::bitbucket::types::{
    get_clone_links, PageResponse, ProjDesc, Project, Repo, RepoUrlBuilder, UserResult,
};
use crate::types::BitBucketOpts;
use crate::util::bail;
use std::time::Duration;

pub type BitbucketResult<T> = std::result::Result<T, BitbucketError>;

pub struct BitbucketError {
    is_timeout: bool,
    msg: String,
    cause: String,
}

pub struct BitbucketWorker<'a> {
    opts: &'a BitBucketOpts,
    timeout_counter: RelaxedCounter,
}

impl BitbucketWorker<'_> {
    pub fn new(opts: &BitBucketOpts) -> BitbucketWorker {
        BitbucketWorker {
            opts,
            timeout_counter: RelaxedCounter::new(0),
        }
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
        match self
            .fetch_all_paginated::<ProjDesc>("projects", "/rest/api/1.0/projects")
            .await
        {
            Ok(all_projects) => Ok(self.fetch_all("projects", all_projects).await?),
            Err(e) if self.opts.verbose => bail(&format!("{}\nCause: {}", e.msg, e.cause))?,
            Err(e) => bail(&e.msg)?,
        }
    }

    pub async fn fetch_all_user_repos(&self) -> Result<Vec<Repo>> {
        match self
            .fetch_all_paginated::<UserResult>("users", "/rest/api/1.0/users")
            .await
        {
            Ok(all_users) => Ok(self.fetch_all("users", all_users).await?),
            Err(e) if self.opts.verbose => bail(&format!("{}\nCause: {}", e.msg, e.cause))?,
            Err(e) => bail(&e.msg)?,
        }
    }

    async fn fetch_all<T>(&self, naming: &str, all_projects: Vec<T>) -> Result<Vec<Repo>>
    where
        T: RepoUrlBuilder,
    {
        let keys: Vec<String> = self
            .opts
            .project_keys
            .iter()
            .map(|k| k.to_lowercase())
            .collect();
        let filtered_projects: Vec<&T> = all_projects
            .iter()
            .filter(|t| keys.is_empty() || keys.contains(&t.get_filter_key()))
            .collect();
        let progress_bar = indicatif::ProgressBar::new(filtered_projects.len() as u64);
        let bar_style = "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} (eta:{eta})";
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!("Fetching {} {}", naming, bar_style))
                .progress_chars("#>-"),
        );
        let fetch_result: Vec<BitbucketResult<Vec<Repo>>> =
            stream::iter(filtered_projects.iter().map(|project| {
                let progress_bar = progress_bar.clone();
                async move {
                    let r = self.fetch_one_project(*project).await;
                    progress_bar.inc(1);
                    r
                }
            }))
            .buffer_unordered(self.opts.concurrency.into())
            .collect::<Vec<BitbucketResult<Vec<Repo>>>>()
            .await;
        progress_bar.finish();
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
        let timeouts = self.timeout_counter.get();
        if timeouts > 0 {
            eprintln!("There were {} timeouts towards bitbucket.", timeouts);
        }
        Ok(all)
    }

    async fn fetch_all_paginated<T>(&self, naming: &str, path: &str) -> BitbucketResult<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let mut host = self.opts.server.clone().unwrap();
        if host.ends_with('/') {
            host.pop();
        }
        let mut start: u32 = 0;
        let mut sum: Vec<T> = vec![];
        'outer: loop {
            let url = format!(
                "{host}{path}?limit=500&start={start}",
                host = host,
                path = path,
                start = start
            );
            for attempt in 1..self.opts.retries + 2 {
                let response: reqwest::Result<reqwest::Response> =
                    self.bake_client(&url).send().await;
                match extract_body::<PageResponse<T>>(response, naming).await {
                    Ok(mut resp) => {
                        sum.append(resp.values.borrow_mut());
                        if resp.is_last_page {
                            break 'outer;
                        } else {
                            start += resp.size;
                            continue 'outer;
                        }
                    }
                    Err(e) if e.is_timeout => {
                        let count: u64 = self.timeout_counter.inc() as u64;
                        if attempt > self.opts.retries {
                            // Last chance blown!
                            return Err(e);
                        } else if let Some(Some(backoff)) =
                            self.opts.backoff_sec.map(|b| b.checked_mul(count + 1))
                        {
                            tokio::time::sleep(Duration::from_secs(backoff)).await;
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            // To be sure we dont escape some case into an endless retry-loop
            return Err(BitbucketError {
                is_timeout: true,
                msg: format!(
                    "Failed to read from bitbucket with {} retries.",
                    self.opts.retries
                ),
                cause: "Timeouts against bitbucket.".to_owned(),
            });
        }
        Ok(sum)
    }

    async fn fetch_one_project<T>(&self, project: &T) -> BitbucketResult<Vec<Repo>>
    where
        T: RepoUrlBuilder,
    {
        let path = project.get_repos_path();
        let projects: Vec<Project> = self.fetch_all_paginated("project", &path).await?;
        let repos = get_clone_links(&projects, self.opts);
        Ok(repos)
    }

    fn bake_client(&self, url: &str) -> RequestBuilder {
        let builder: RequestBuilder = ReqwestClient::new()
            .get(url)
            .timeout(Duration::from_secs(self.opts.timeout_sec))
            .header(ACCEPT, "application/json");
        match (&self.opts.username, &self.opts.password) {
            (Some(u), Some(p)) => builder.basic_auth(u, Some(p)),
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
            Err(e) if e.is_timeout() => Err(BitbucketError {
                is_timeout: true,
                msg: "Timeout reading from bitbucket.".to_owned(),
                cause: format!("{:?}", e),
            }),
            Err(e) => Err(BitbucketError {
                is_timeout: e.is_timeout(),
                msg: format!(
                    "Failed fetching {} from bitbucket, bad json format.",
                    naming
                ),
                cause: format!("{:?}", e),
            }),
        },
        Ok(response) => Err(BitbucketError {
            is_timeout: false,
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
            is_timeout: e.is_timeout(),
            msg: format!("Failed fetching {} from bitbucket.", naming),
            cause: format!("{:?}", e),
        }),
    }
}

#[cfg(test)]
mod tests {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    use crate::bitbucket::types::ProjDesc;
    use crate::types::CloneType;

    use super::*;

    fn random_string(len: usize) -> String {
        let mut rng = thread_rng();
        let chars: String = std::iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(len)
            .collect();
        chars
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
                host = random_string(12),
                path = random_string(12)
            )),
            clone_type: CloneType::HTTP,
            project_keys: vec!["key".to_owned()],
            all: false,
            timeout_sec: 10,
            retries: 0,
            backoff_sec: None,
        }
    }

    #[tokio::test]
    async fn fetch_one_bad_host_is_dns_error() {
        // given
        let project: ProjDesc = ProjDesc {
            key: "key".to_owned(),
        };
        let bit_bucket_opts = basic_opts();
        let worker = BitbucketWorker::new(&bit_bucket_opts);

        // when
        let result = worker.fetch_one_project(&project).await;

        // then
        match result {
            Ok(_) => panic!("This request was expected to fail."),
            Err(e) => {
                assert_eq!(
                    e.msg, "Failed fetching project from bitbucket.",
                    "Unexpected error message. Was '{}'",
                    e.msg
                );
                assert!(
                    e.cause.contains("ConnectError(\"dns error\""),
                    "Was {}",
                    e.cause
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
        let worker = BitbucketWorker::new(&bit_bucket_opts);

        // when
        let result = worker.fetch_one_project(&project).await;

        // then
        match result {
            Ok(_) => panic!("This request was expected to fail."),
            Err(e) => assert!(
                e.msg.contains("status code: 404"),
                "Response code should be 404, but was {:?}",
                e.cause
            ),
        }
    }
}
