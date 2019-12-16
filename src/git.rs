use std::path::Path;
use std::result::Result as StdResult;

use generic_error::{GenericError, Result};
use git2::{Branch, FetchOptions, MergeOptions, RemoteCallbacks, Repository, RepositoryState};
use git2::BranchType::Local;
use git2::build::{CheckoutBuilder, RepoBuilder};
use rayon::prelude::*;

use crate::types::Opts;

use super::types::Repo;

pub fn git_going(opts: &Opts, repos: Vec<Repo>) {
    println!("Started working {} repositories", repos.len());
    let pool = rayon::ThreadPoolBuilder::new().num_threads(opts.thread_count).build().unwrap();
    pool.install(|| {
        repos.into_par_iter().for_each(|repo| {
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
            clone_or_update(&repo, cb);
        });
    });
    println!();
    println!("Done");
}

fn clone_or_update(repo: &Repo, cb: RemoteCallbacks) {
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    if dir_exists(&repo) {
        git_update(&repo, fo);
    } else {
        git_clone( &repo, fo);
    }
}

fn dir_exists(repo: &Repo) -> bool {
    return match std::fs::read_dir(Path::new(&format!("./{}/{}", repo.project_key, repo.name)[..])) {
        Ok(_) => true,
        _ => false,
    };
}

fn git_clone(repo: &Repo, fo: FetchOptions) {
    let s = format!("./{}/{}", repo.project_key.clone(), repo.name.clone());
    let p = Path::new(s.trim());
    match RepoBuilder::new().fetch_options(fo).clone(&repo.git, p) {
        Ok(_) => println!("Cloned {}/{}", repo.project_key, repo.name),
        Err(e) => eprintln!("Failed cloning {}/{} {:?}", repo.project_key, repo.name, e),
    }
}

fn git_update(repo: &Repo, fo: FetchOptions) {
    let s = format!("./{}/{}/.git", repo.project_key.clone(), repo.name.clone());
    let p = Path::new(s.trim());
    let repository = match Repository::open(p) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed opening repo {}/{} {:?}", repo.project_key, repo.name, e);
            return;
        }
    };
    match git_update_repository( &repository, fo) {
        Ok(_) => println!("Updated {}/{}", repo.project_key, repo.name),
        Err(e) => {
            if e.msg.trim() == "Not Clean repo state" {
                eprintln!("Failed updating {}/{} {}", repo.project_key, repo.name, e.msg)
            } else {
                eprintln!("Failed updating {}/{} {}", repo.project_key, repo.name, e.msg);
                match repository.cleanup_state() {
                    Ok(_) => eprintln!("Failed updating, but reset {}/{} {}", repo.project_key, repo.name, e.msg),
                    Err(e2) => eprintln!("Failed updating, failed reset {}/{} update_error:{:?} reset_error:{:?}",
                                         repo.project_key, repo.name, e, e2)
                }
            }
        },
    }
}

fn git_update_repository(repo: &Repository, mut fo: FetchOptions) -> Result<()> {
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => produce_error("Failed find remote origin")?,
    };
    match remote.fetch(&["--all"], Some(&mut fo), None) {
        Ok(_) => {},
        Err(_) => produce_error("Failed fetch --all")?,
    }
    match repo.state() {
        RepositoryState::Clean => {},
        e => produce_error(format!("Not Clean repo state {:?}", e).trim())?,
    }
    let head = repo.head()?;
    let curr_branch_name = match head.shorthand(){
        Some(shorthand) => shorthand,
        None => produce_error("Not on a branch")?,
    };

    let branch: Branch = repo.find_branch(curr_branch_name, Local)?;
    let origin_branch = branch.upstream()?;
    let branch_reference = branch.into_reference();
    let branch_commit = repo.reference_to_annotated_commit(&branch_reference)?;
    let head_commit = repo.reference_to_annotated_commit(&(repo.head()?))?;
    if branch_commit.id() != head_commit.id() {
        produce_error(format!("Head is not on same place as {}.", curr_branch_name).trim())?;
    }
    let origin_commit = repo.reference_to_annotated_commit(&origin_branch.into_reference())?;

    repo.merge(&[&origin_commit],
               Some(MergeOptions::default().fail_on_conflict(true)),
               Some(CheckoutBuilder::default().force()))?;
    Ok(())
}

fn  produce_error<T>(msg: &str) -> Result<T>{
    return Err(GenericError { msg: msg.to_string() });
}
