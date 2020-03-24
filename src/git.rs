use std::io::Write;
use std::path::Path;
use std::process::{Command, Output};
use std::result::Result as StdResult;

use generic_error::Result;
use git2::{FetchOptions, RemoteCallbacks};
use git2::build::RepoBuilder;
use rayon::prelude::*;

use crate::types::{Opts, Repo};

pub fn git_going(opts: &Opts, repos: Vec<Repo>) {
    println!("Started working {} repositories", repos.len());
    println!("Progress: ... c=Clone U=Updated, u=Already up to date, e=Err");
    let pool = rayon::ThreadPoolBuilder::new().num_threads(opts.thread_count).build().unwrap();
    let failed: Vec<String> = pool.install(|| {
        repos.into_par_iter().map(|repo| {
            let mut cb = RemoteCallbacks::new();
            cb.credentials(|_user: &str, _user_from_url: Option<&str>, _cred: git2::CredentialType, | -> StdResult<git2::Cred, git2::Error> {
                let user = _user_from_url.unwrap_or("git");
                let home = std::env::var("HOME").unwrap();
                let private_key = format!("{}/.ssh/id_rsa", home);
                let pub_key = format!("{}.pub", private_key);
                git2::Cred::ssh_key(user, Some(Path::new(&pub_key)), Path::new(&private_key), None)
            });
            match clone_or_update(&repo, &opts, cb) {
                Ok(result) => {
                    print!("{}", result);
                    std::io::stdout().flush().unwrap_or(());
                    None
                }
                Err(e) => {
                    print!("e");
                    std::io::stdout().flush().unwrap_or(());
                    Some(format!("{}/{} error:{}", repo.project_key, repo.name, e.msg))
                }
            }
        }).filter_map(|result: Option<String>| result).collect()
    });

    if !failed.is_empty() {
        eprintln!("\n{} projects failed to update or clone.", failed.len());
        for fail in failed {
            eprintln!("{}", fail);
        }
    } else {
        println!();
    }
    println!("Done");
}

fn clone_or_update<'a>(repo: &'a Repo, opts: &Opts, cb: RemoteCallbacks) -> Result<&'a str> {
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    match dir_exists(&repo) {
        true => {
            if opts.reset_state {
                git_reset(repo)?;
            }
            Ok(git_update(&repo)?)
        }
        false => {
            git_clone(&repo, fo)?;
            git_reset(repo)?;
            Ok(&"c")
        }
    }
}

fn git_clone(repo: &Repo, fo: FetchOptions) -> Result<()> {
    let s = path(&repo);
    let path = Path::new(&s);
    RepoBuilder::new().fetch_options(fo).clone(&repo.git, path)?;
    Ok(())
}

fn git_update(repo: &Repo) -> Result<&str> {
    let out = exec("git pull --ff-only", &repo)?;
    let success = out.status.success();
    let output = format!("{:?}", std::str::from_utf8(out.stdout.as_slice()));
    return if success {
        if output.contains(&"Already up to date.") {
            Ok("u")
        } else {
            Ok("U")
        }
    } else {
        Ok("e")
    }
}

fn git_reset(repo: &Repo) -> Result<()> {
    exec("git reset --hard", repo)?;
    exec("git checkout master --quiet --force --theirs", repo)?;
    Ok(())
}

fn exec(cmd: &str, repo: &Repo) -> Result<Output> {
    let is_win: bool = cfg!(target_os = "windows");
    let string_path = path(&repo);
    let path = Path::new(&string_path);
    return match is_win {
        true => {
            Ok(Command::new("cmd").args(&["/C", cmd]).current_dir(path).output()?)
        },
        false => {
            Ok(Command::new("sh").args(&["-c", cmd]).current_dir(path).output()?)
        }
    };
}

fn path(repo: &Repo) -> String {
    return format!("./{}/{}", repo.project_key.clone(), repo.name.clone());
}

fn dir_exists(repo: &Repo) -> bool {
    return match std::fs::read_dir(Path::new(&format!("./{}/{}", repo.project_key, repo.name)[..])) {
        Ok(_) => true,
        _ => false,
    };
}
