use std::path::Path;
use std::process::Output;

use futures::stream::{self, StreamExt};
use generic_error::{GenericError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::process::Command;

use crate::types::{GitOpts, Repo};

#[derive(Clone)]
pub struct Git {
    pub repos: Vec<Repo>,
    pub opts: GitOpts,
}

impl Git {
    pub async fn git_going(self) {
        println!("Started working {} repositories", self.repos.len());
        let bar: ProgressBar = ProgressBar::new(self.repos.len() as u64);
        bar.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} (eta:{eta})")
            .progress_chars("#>-"));
        let mut projects: Vec<String> = self.repos.iter().map(|r| r.project_key.clone()).collect();
        projects.dedup();
        projects.iter().for_each(|p| match std::fs::create_dir(p) {
            Ok(_) => {},
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(e) => panic!("{:?}", e),
        });
        let clone_result = stream::iter(
            self.repos.iter().map(|repo| {
                let reset = self.opts.reset_state;
                let bar = bar.clone();
                async move {
                    let result = clone_or_update(&repo, reset).await;
                    bar.inc(1);
                    result
                }
            })
        ).buffer_unordered(self.opts.concurrency).collect::<Vec<Result<()>>>().await;
        bar.finish();
        let mut failed: Vec<String> = vec![];
        for result in clone_result {
            match result {
                Err(e) => failed.push(e.msg),
                _ => {},
            }
        }

        if !failed.is_empty() {
            eprintln!("\n{} projects failed to update or clone.", failed.len());
            if !self.opts.quiet {
                for fail in failed {
                    eprintln!("{}", fail);
                }
            }
        }
    }
}

async fn clone_or_update(repo: &Repo, do_reset_state: bool) -> Result<()> {
    match dir_exists(&repo) {
        true => {
            if do_reset_state {
                git_reset(repo).await?;
            }
            git_update(&repo).await?;
        }
        false => {
            git_clone(&repo).await?;
            git_reset(repo).await?;
        }
    }
    Ok(())
}

async fn git_clone(repo: &Repo) -> Result<()> {
    let string_path = format!("./{}", repo.project_key);
    let path = Path::new(&string_path);

    let fail_suffix = "failed git clone";
    match exec(&*format!("git clone {} {}", repo.git, repo.name), path).await {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) if !o.status.success() => Err(generate_repo_err_from_output(fail_suffix, repo, o.stdout, o.stderr)),
        Err(e) => Err(generate_repo_err(fail_suffix, repo, e.msg)),
        _ => Err(generate_repo_err(fail_suffix, repo, "unknown".to_owned()))
    }
}

async fn git_update(repo: &Repo) -> Result<()> {
    let string_path = path(&repo);
    let path = Path::new(&string_path);

    let fail_suffix = "failed git pull";
    match exec("git pull --ff-only", path).await {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) if !o.status.success() => Err(generate_repo_err_from_output(fail_suffix, repo, o.stdout, o.stderr)),
        Err(e) => Err(generate_repo_err(fail_suffix, repo, e.msg)),
        _ => Err(generate_repo_err(fail_suffix, repo, "unknown".to_owned()))
    }
}

async fn git_reset(repo: &Repo) -> Result<()> {
    let string_path = path(repo);
    let path = Path::new(&string_path);
    match exec("git reset --hard", path).await {
        Ok(_) => match exec("git checkout master --quiet --force --theirs", path).await {
            Err(e) => Err(generate_repo_err("failed 'checkout master'", repo, e.msg)),
            Ok(_) => Ok(()),
        },
        Err(e) => Err(generate_repo_err("failed resetting repo", repo, e.msg))
    }
}

fn generate_repo_err_from_output(suffix: &str, repo: &Repo, cause_out: Vec<u8>, cause_err: Vec<u8>) -> GenericError {
    let cause = match (cause_to_str(cause_err), cause_to_str(cause_out)) {
        (Some(e), Some(o)) => format!("Err: '{}' Output: '{}'", e.trim(), o.trim()),
        (Some(e), None) => format!("Err: '{}'", e.trim()),
        (None, Some(o)) => format!("Output: '{}'", o.trim()),
        (None, None) => format!("no output"),
    };
    generate_repo_err(suffix, repo, cause)
}

fn cause_to_str(cause: Vec<u8>) -> Option<String> {
    if cause.is_empty() {
        return None;
    }
    match std::str::from_utf8(cause.as_slice()) {
        Ok(cause) => Some(cause.to_owned()),
        _ => None
    }
}

fn generate_repo_err(suffix: &str, repo: &Repo, cause: String) -> GenericError {
    GenericError { msg: format!("{}/{} {}. Cause: {}", repo.project_key, repo.name, suffix, cause) }
}

async fn exec(cmd: &str, path: &Path) -> Result<Output> {
    let is_win: bool = cfg!(target_os = "windows");
    match is_win {
        true => {
            Ok(Command::new("cmd").args(&["/C", cmd]).current_dir(path).output().await?)
        },
        false => {
            Ok(Command::new("sh").args(&["-c", cmd]).current_dir(path).output().await?)
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(project_key: &str, name: &str) -> Repo {
        Repo {
            project_key: String::from(project_key),
            git: String::from("https://github.com/jensim/bitbucket_server_cli.git"),
            name: String::from(name),
        }
    }

    #[tokio::test]
    async fn test_git_clone_and_update() {
        let path = "tmp/test_repo";
        let repo = repo("tmp", "test_repo");
        std::fs::remove_dir_all(Path::new(path)).unwrap_or(());
        match std::fs::read_dir(path) {
            Ok(_) => assert!(false, "Failed cleaning away dir."),
            Err(_e) => {},
        }
        std::fs::create_dir("tmp").unwrap_or(());

        git_clone(&repo).await.unwrap();
        match std::fs::read_dir(path) {
            Ok(_) => {},
            Err(e) => assert!(false, "Failed. {:?}", e),
        }

        git_update(&repo).await.unwrap();
        match std::fs::read_dir(path) {
            Ok(_) => {},
            Err(e) => assert!(false, "Failed. {:?}", e),
        }
    }
}
