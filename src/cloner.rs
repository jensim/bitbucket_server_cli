use generic_error::Result;

use crate::{
    bitbucket::Bitbucket,
    git::Git,
    input::select_projects,
    types::{CloneOpts, Repo},
};

pub struct Cloner {
    opts: CloneOpts,
}

impl Cloner {
    pub fn new(opts: CloneOpts) -> Cloner {
        let mut opts = opts;
        opts.validate();
        Cloner { opts }
    }

    pub async fn git_clone(self) -> Result<()> {
        let mut project_keys = self.opts.git_opts.project_keys();

        let bb = Bitbucket {
            opts: self.opts.bitbucket_opts.clone(),
        };
        let mut repos: Vec<Repo> = match bb.fetch_all().await {
            Ok(r) => r,
            Err(e) => {
                println!("Failed getting password from env. {}", e.msg);
                std::process::exit(1);
            }
        };
        if self.opts.interactive() && !self.opts.git_opts.clone_all && project_keys.is_empty() {
            project_keys = select_projects(&repos);
        }

        if !self.opts.git_opts.clone_all && !project_keys.is_empty() {
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
    #[ignore] // TODO fix test
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
            },
            git_opts: GitOpts {
                clone_all: true,
                project_keys: vec![],
                reset_state: false,
                concurrency: 1,
                quiet: false,
                output_directory: ".".to_owned(),
            },
        };
        match Cloner::new(opts).git_clone().await {
            Ok(_) => assert!(
                false,
                "GitHub.com should never be available as a bitbucket server"
            ),
            Err(e) => println!("{}", e.msg),
        }
    }
}
