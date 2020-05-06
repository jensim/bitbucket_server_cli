use generic_error::Result;

use crate::util::bail;
use crate::{
    bitbucket::types::Repo, bitbucket::worker::BitbucketWorker, git::Git, input::select_projects,
    types::CloneOpts,
};

pub struct Cloner {
    opts: CloneOpts,
}

impl Cloner {
    pub fn new(opts: CloneOpts) -> Result<Cloner> {
        let mut opts = opts;
        opts.validate()?;
        Ok(Cloner { opts })
    }

    pub async fn clone_projects_and_users(self) -> Result<()> {
        let bb = BitbucketWorker::new(self.opts.bitbucket_opts.clone());
        let repos: Vec<Repo> = match bb.fetch_all_repos().await {
            Ok(r) => r,
            Err(e) => bail(&format!("Failed fetching user & project repos. {}", e.msg))?,
        };
        self.clone_repos(repos).await?;
        Ok(())
    }

    pub async fn clone_projects(self) -> Result<()> {
        let bb = BitbucketWorker::new(self.opts.bitbucket_opts.clone());
        let repos: Vec<Repo> = match bb.fetch_all_project_repos().await {
            Ok(r) => r,
            Err(e) => bail(&format!("Failed fetching project repos. {}", e.msg))?,
        };
        self.clone_repos(repos).await?;
        Ok(())
    }

    pub async fn clone_users(self) -> Result<()> {
        let bb = BitbucketWorker::new(self.opts.bitbucket_opts.clone());
        let repos: Vec<Repo> = match bb.fetch_all_user_repos().await {
            Ok(r) => r,
            Err(e) => bail(&format!("Failed fetching user repos. {}", e.msg))?,
        };
        self.clone_repos(repos).await?;
        Ok(())
    }

    async fn clone_repos(self, mut repos: Vec<Repo>) -> Result<()> {
        let mut project_keys = self.opts.bitbucket_opts.project_keys();
        if self.opts.interactive() && !self.opts.bitbucket_opts.all && project_keys.is_empty() {
            project_keys = select_projects(&repos);
        }

        if !self.opts.bitbucket_opts.all && !project_keys.is_empty() {
            let mut tmp_vec = Vec::new();
            for r in repos {
                if project_keys.contains(&r.project_key) {
                    tmp_vec.push(r);
                }
            }
            repos = tmp_vec;
        }
        Git {
            opts: self.opts.git_opts,
            repos,
        }
        .git_going()
        .await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{BitBucketOpts, CloneType, GitOpts};

    use super::*;

    #[tokio::test]
    async fn cloner_integration_test() {
        let opts = CloneOpts {
            batch_mode: true,
            bitbucket_opts: BitBucketOpts {
                server: Some("http://github.com".to_owned()),
                verbose: true,
                concurrency: 1,
                password: Some("PA$$WoRD123#%&".to_owned()),
                password_from_env: false,
                username: Some("Admin".to_owned()),
                clone_type: CloneType::HTTP,
                project_keys: vec![],
                all: true,
            },
            git_opts: GitOpts {
                reset_state: false,
                concurrency: 1,
                quiet: false,
                output_directory: ".".to_owned(),
            },
        };
        match Cloner::new(opts).unwrap().clone_projects().await {
            Ok(_) => assert!(
                false,
                "GitHub.com should never be available as a bitbucket server"
            ),
            Err(e) => println!("{}", e.msg),
        }
    }
}
