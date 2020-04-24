use std::{
    path::Path,
    process::{Command, Output},
};

use generic_error::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rayon::ThreadPool;

use crate::types::{Opts, Repo};

pub fn git_going(opts: &Opts, repos: &Vec<Repo>) {
    println!("Started working {} repositories", repos.len());
    let bar: ProgressBar = ProgressBar::new(repos.len() as u64);
    bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} (eta:{eta})")
        .progress_chars("#>-"));
    let mut projects: Vec<String> = repos.iter().map(|r| r.project_key.clone()).collect();
    projects.dedup();
    projects.iter().for_each(|p| match std::fs::create_dir(p) {
        Ok(_) => {},
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {},
        Err(e) => panic!("{:?}", e),
    });

    let pool: ThreadPool = rayon::ThreadPoolBuilder::new().num_threads(opts.thread_count).build().unwrap();
    let failed: Vec<String> = pool.install(|| {
        repos.into_par_iter().map(|repo| {
            let result = match clone_or_update(&repo, &opts) {
                Ok(_result) => None,
                Err(e) => Some(format!("{}/{} error:{}", repo.project_key, repo.name, e.msg))
            };
            bar.inc(1);
            result
        }).filter_map(|result: Option<String>| result).collect()
    });
    bar.finish();

    if !failed.is_empty() {
        eprintln!("\n{} projects failed to update or clone.", failed.len());
        for fail in failed {
            eprintln!("{}", fail);
        }
    }
}

fn clone_or_update<'a>(repo: &'a Repo, opts: &Opts) -> Result<&'a str> {
    match dir_exists(&repo) {
        true => {
            if opts.reset_state {
                git_reset(repo)?;
            }
            Ok(git_update(&repo)?)
        }
        false => {
            git_clone(&repo)?;
            git_reset(repo)?;
            Ok(&"c")
        }
    }
}

fn git_clone(repo: &Repo) -> Result<&str> {
    let string_path = format!("./{}", repo.project_key);
    let path = Path::new(&string_path);
    let out = exec(&*format!("git clone {}", repo.git), path)?;

    let success = out.status.success();
    return match success {
        true => Ok("c"),
        false => Ok("e")
    }
}

fn git_update(repo: &Repo) -> Result<&str> {
    let string_path = path(&repo);
    let path = Path::new(&string_path);
    let out = exec("git pull --ff-only", path)?;
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
    let string_path = path(repo);
    let path = Path::new(&string_path);
    exec("git reset --hard", path)?;
    exec("git checkout master --quiet --force --theirs", path)?;
    Ok(())
}

fn exec(cmd: &str, path: &Path) -> Result<Output> {
    let is_win: bool = cfg!(target_os = "windows");
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
