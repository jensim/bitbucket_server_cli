use std::io::{Write, Read};
use std::path::Path;
use std::process::{Child, Command};
use std::result::Result as StdResult;

use generic_error::{GenericError, Result};
use git2::{FetchOptions, RemoteCallbacks};
use git2::build::RepoBuilder;
use rayon::prelude::*;

use crate::types::{Opts, Repo};

pub fn git_going(opts: &Opts, repos: Vec<Repo>) {
    println!("Started working {} repositories", repos.len());
    println!("Progress: ... c=Clone _=Exists");
    let pool = rayon::ThreadPoolBuilder::new().num_threads(opts.thread_count).build().unwrap();
    let failed: Vec<String> = pool.install(|| {
        repos.into_par_iter().map(|repo| {
            let mut cb = RemoteCallbacks::new();
            cb.credentials(|_user: &str, _user_from_url: Option<&str>, _cred: git2::CredentialType, | -> StdResult<git2::Cred, git2::Error> {
                let user = _user_from_url.unwrap_or("git");
                let home = std::env::var("HOME").unwrap();
                let private_key = format!("{}/.ssh/id_rsa", home);
                let pub_key = format!("{}.pub", private_key);
                let ssh_pass = opts.git_ssh_password.clone();
                let pass = ssh_pass.as_ref().map(|p| p.trim());
                git2::Cred::ssh_key(user, Some(Path::new(&pub_key)), Path::new(&private_key), pass)
            });
            match clone_or_update(opts.clone(), &repo, cb) {
                Ok(result) => None,
                Err(e) => Some(format!("{}/{} error:{}", repo.project_key, repo.name, e.msg)),
            }
        }).filter_map(|result: Option<String>| result).collect()
    });

    if !failed.is_empty() {
        eprintln!("{} projects failed to update or clone.", failed.len());
        for fail in failed {
            eprintln!("{}", fail);
        }
    }else {
        println!();
    }
    println!("Done");
    println!("To perform update try out this:");
    println!("alias git-pull-recursive='find . -maxdepth 3 -mindepth 2 -type d -name .git -exec sh -c \"cd \\\"{}\\\"/../ && git reset --hard -q && git pull -q --ff-only &\" \\;'", "{}");
}

fn clone_or_update(opts: &Opts, repo: &Repo, cb: RemoteCallbacks) -> Result<()> {
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    match dir_exists(&repo) {
        true => {
            print!("_");
            std::io::stdout().flush()?;
            Ok(())
        }
        false => {
            git_clone(&repo, fo)?;
            print!("c");
            std::io::stdout().flush()?;
            Ok(())
        }
    }
}

fn git_clone(repo: &Repo, fo: FetchOptions) -> Result<()> {
    let s = format!("./{}/{}", repo.project_key.clone(), repo.name.clone());
    let p = Path::new(s.trim());
    RepoBuilder::new().fetch_options(fo).clone(&repo.git, p)?;
    Ok(())
}

fn dir_exists(repo: &Repo) -> bool {
    return match std::fs::read_dir(Path::new(&format!("./{}/{}", repo.project_key, repo.name)[..])) {
        Ok(_) => true,
        _ => false,
    };
}
