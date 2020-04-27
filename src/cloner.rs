use generic_error::Result;

use crate::{
    bitbucket::Bitbucket,
    git::Git,
    input::{
        self,
        password_from_env,
        select_projects,
    },
    types::{
        Opts,
        Repo,
    },
};

pub struct Cloner {
    opts: Opts,
}

impl Cloner {
    pub fn new(opts: Opts) -> Cloner {
        Cloner { opts }
    }

    pub async fn clone(self) -> Result<()> {
        let mut opts = self.opts;

        if opts.interactive {
            opts = input::opts(&opts);
        } else if opts.bitbucket_opts.server.is_none() {
            println!("Server is required");
            std::process::exit(1);
        } else if !opts.git_opts.clone_all && opts.git_opts.project_keys.is_empty() {
            println!("project selection is required (all or keys)");
            std::process::exit(1);
        } else if opts.git_opts.concurrency > 100 || opts.bitbucket_opts.concurrency > 100 {
            println!("Max concurrent actions = 100");
            std::process::exit(1);
        }
        if opts.bitbucket_opts.password_from_env {
            match password_from_env() {
                Ok(pass) => opts.bitbucket_opts.password = Some(pass),
                Err(e) => {
                    println!("Failed getting env password. {}", e.msg);
                    std::process::exit(1);
                }
            }
        }
        let bb = Bitbucket { opts: opts.bitbucket_opts.clone() };
        let mut repos: Vec<Repo> = match bb.fetch_all().await {
            Ok(r) => r,
            Err(e) => {
                println!("Failed getting password from env. {}", e.msg);
                std::process::exit(1);
            }
        };
        if opts.interactive && !opts.git_opts.clone_all && opts.git_opts.project_keys.is_empty() {
            opts.git_opts.project_keys = select_projects(&repos);
        }

        if !opts.git_opts.clone_all && !opts.git_opts.project_keys.is_empty() {
            let mut tmp_vec = Vec::new();
            for r in repos {
                if opts.git_opts.project_keys.contains(&r.project_key) {
                    tmp_vec.push(r);
                }
            }
            repos = tmp_vec;
        }
        Git { opts: opts.git_opts, repos }.git_going().await;
        Ok(())
    }
}
