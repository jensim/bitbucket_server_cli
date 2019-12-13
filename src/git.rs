use generic_error::{GenericError, Result};
use rayon::prelude::*;
use std::result::Result as StdResult;

use super::types::Repo;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{FetchOptions, Progress, RemoteCallbacks};

struct State {
    progress: Option<Progress<'static>>,
    total: usize,
    current: usize,
    path: Option<PathBuf>,
    newline: bool,
}

pub fn git_going(repos: Vec<Repo>) {
    repos.into_par_iter().for_each(|repo| clone_or_update(repo));
}

fn clone_or_update(repo: Repo) {
    if dir_exists(&repo) {
        // update(&repo);
        eprintln!("Repo exists already {}/{}", &repo.project_key, &repo.name);
    } else {
        clone(&repo)
    }
}

fn dir_exists(repo: &Repo) -> bool {
    return match std::fs::read_dir(Path::new(&format!("./{}/{}", repo.project_key, repo.name)[..])) {
        Ok(_) => true,
        _ => false,
    };
}

fn clone(repo: &Repo) {
    let state = RefCell::new(State {
        progress: None,
        total: 0,
        current: 0,
        path: None,
        newline: false,
    });
    let mut cb = RemoteCallbacks::new();
    cb.transfer_progress(|stats| {
        let mut state = state.borrow_mut();
        state.progress = Some(stats.to_owned());
        // print(&mut *state);
        true
    });
    cb.credentials(git_credentials_callback);

    let mut co = CheckoutBuilder::new();
    co.progress(|path, cur, total| {
        let mut state = state.borrow_mut();
        state.path = path.map(|p| p.to_path_buf());
        state.current = cur;
        state.total = total;
        // print(&mut *state);
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    println!("Clone {:?}", repo);
    let s = format!("./{}/{}", repo.project_key.clone(), repo.name.clone());
    let p = Path::new(s.trim());
    let clone_result = RepoBuilder::new()
        .fetch_options(fo)
        .with_checkout(co)
        .clone(&repo.git, p);
    match clone_result {
        Ok(_) => {
            println!("Cloned {}/{}", &repo.project_key, &repo.name);
        }
        Err(e) => {
            eprintln!("Failed cloning repo {}/{} {:?}", &repo.project_key, &repo.name, e);
        }
    }

    fn git_credentials_callback(
        _user: &str,
        _user_from_url: Option<&str>,
        _cred: git2::CredentialType,
    ) -> StdResult<git2::Cred, git2::Error> {

        let user = _user_from_url.unwrap_or("git");

        if _cred.contains(git2::CredentialType::USERNAME) {
            return git2::Cred::username(user);
        }

        let home = std::env::var("HOME").unwrap();
        let private_key = format!("{}/.ssh/id_rsa", home);
        let pub_key = format!("{}.pub", private_key);
        git2::Cred::ssh_key(user, Some(Path::new(&pub_key)), Path::new(&private_key), None)
    }
}
