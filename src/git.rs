use std::path::{Path};
use std::result::Result as StdResult;

use git2::{FetchOptions, RemoteCallbacks};
use git2::build::{RepoBuilder};
use rayon::prelude::*;

use crate::types::Opts;

use super::types::Repo;
use std::process::Child;

pub fn git_going(opts: &Opts, repos: Vec<Repo>) {
    repos.into_par_iter().for_each(|repo| clone_or_update(&opts, &repo));
}

fn clone_or_update(opts: &Opts, repo: &Repo) {
    if dir_exists(&repo) {
        update(&repo);
    } else {
        clone(opts, &repo);
    }
}

fn dir_exists(repo: &Repo) -> bool {
    return match std::fs::read_dir(Path::new(&format!("./{}/{}", repo.project_key, repo.name)[..])) {
        Ok(_) => true,
        _ => false,
    };
}

fn update(repo: &Repo){
    let s = format!("./{}/{}", repo.project_key.clone(), repo.name.clone());
    let p = Path::new(s.trim());
    let result: std::io::Result<Child> = std::process::Command::new("sh")
        .arg("-C")
        .arg(p)
        .arg("git")
        .arg("pull")
        .spawn();
    match result {
        Ok (mut e) => {
            match e.wait() {
                Ok(_) => {
                    println!("Repo updated {}/{}", &repo.project_key, &repo.name);
                },
                Err(e) => {
                    eprintln!("Failed updating repo {} {:?}", s, e)
                },
            }
        },
        Err(e)  => {
            eprintln!("Failed updating repo {} {:?}", s, e)
        },
    }
}

fn clone(opts: &Opts, repo: &Repo) {
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

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    println!("Clone {:?}", repo);
    let s = format!("./{}/{}", repo.project_key.clone(), repo.name.clone());
    let p = Path::new(s.trim());
    match RepoBuilder::new().fetch_options(fo).clone(&repo.git, p) {
        Ok(_) => {
            println!("Cloned {}/{}", &repo.project_key, &repo.name);
        }
        Err(e) => {
            eprintln!("Failed cloning repo {}/{} {:?}", &repo.project_key, &repo.name, e);
        }
    }
}
