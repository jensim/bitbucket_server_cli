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
        if self.repos.is_empty() {
            eprintln!("No repos to work on");
            return;
        }
        println!("Started working {} repositories", self.repos.len());
        let progress_bar: ProgressBar = ProgressBar::new(self.repos.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} (eta:{eta})")
                .progress_chars("#>-"),
        );
        let mut projects: Vec<String> = self.repos.iter().map(|r| r.project_key.clone()).collect();
        projects.dedup();
        projects.iter().for_each(|p| {
            match std::fs::create_dir_all(format!("{}/{}", self.opts.output_directory, p)) {
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => {
                    eprintln!("Unable to create project dir {} due to err: {:?}", p, e);
                    std::process::exit(1);
                }
            }
        });
        let clone_result = stream::iter(self.repos.iter().map(|repo| {
            let opts = self.opts.clone();
            let progress_bar = progress_bar.clone();
            async move {
                let result = clone_or_update(&repo, opts).await;
                progress_bar.inc(1);
                result
            }
        }))
        .buffer_unordered(self.opts.concurrency)
        .collect::<Vec<Result<()>>>()
        .await;

        progress_bar.finish();
        let mut failed: Vec<String> = vec![];
        for result in clone_result {
            if let Err(e) = result {
                failed.push(e.msg);
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

async fn clone_or_update(repo: &Repo, git_opts: GitOpts) -> Result<()> {
    if dir_exists(&repo, &git_opts.output_directory) {
        if git_opts.reset_state {
            git_reset(repo, &git_opts.output_directory).await?;
        }
        git_update(&repo, &git_opts.output_directory).await?;
    } else {
        git_clone(&repo, &git_opts.output_directory).await?;
        git_reset(repo, &git_opts.output_directory).await?;
    }
    Ok(())
}

async fn git_clone(repo: &Repo, output_directory: &str) -> Result<()> {
    let string_path = format!("{}/{}", output_directory, repo.project_key);
    let path = Path::new(&string_path);

    let fail_suffix = format!("failed git clone into {}", output_directory);
    match exec(&*format!("git clone {} {}", repo.git, repo.name), path).await {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) if !o.status.success() => Err(generate_repo_err_from_output(
            &fail_suffix,
            repo,
            o.stdout,
            o.stderr,
        )),
        Err(e) => Err(generate_repo_err(&fail_suffix, repo, e.msg)),
        _ => Err(generate_repo_err(&fail_suffix, repo, "unknown".to_owned())),
    }
}

async fn git_update(repo: &Repo, output_directory: &str) -> Result<()> {
    let string_path = path(&repo, output_directory);
    let path = Path::new(&string_path);

    let fail_suffix = "failed git pull";
    match exec("git pull --ff-only", path).await {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) if !o.status.success() => Err(generate_repo_err_from_output(
            fail_suffix,
            repo,
            o.stdout,
            o.stderr,
        )),
        Err(e) => Err(generate_repo_err(fail_suffix, repo, e.msg)),
        _ => Err(generate_repo_err(fail_suffix, repo, "unknown".to_owned())),
    }
}

async fn git_reset(repo: &Repo, output_directory: &str) -> Result<()> {
    let string_path = path(repo, output_directory);
    let path = Path::new(&string_path);
    match exec("git reset --hard", path).await {
        Ok(_) => match exec("git checkout master --quiet --force --theirs", path).await {
            Err(e) => Err(generate_repo_err("failed 'checkout master'", repo, e.msg)),
            Ok(_) => Ok(()),
        },
        Err(e) => Err(generate_repo_err("failed resetting repo", repo, e.msg)),
    }
}

fn generate_repo_err_from_output(
    suffix: &str,
    repo: &Repo,
    cause_out: Vec<u8>,
    cause_err: Vec<u8>,
) -> GenericError {
    let cause = match (cause_to_str(cause_err), cause_to_str(cause_out)) {
        (Some(e), Some(o)) => format!("Err: '{}' Output: '{}'", e.trim(), o.trim()),
        (Some(e), None) => format!("Err: '{}'", e.trim()),
        (None, Some(o)) => format!("Output: '{}'", o.trim()),
        (None, None) => "no output".to_string(),
    };
    generate_repo_err(suffix, repo, cause)
}

fn cause_to_str(cause: Vec<u8>) -> Option<String> {
    if cause.is_empty() {
        None
    } else if let Ok(cause) = std::str::from_utf8(cause.as_slice()) {
        Some(cause.to_owned())
    } else {
        None
    }
}

fn generate_repo_err(suffix: &str, repo: &Repo, cause: String) -> GenericError {
    GenericError {
        msg: format!(
            "{}/{} {}. Cause: {}",
            repo.project_key, repo.name, suffix, cause
        ),
    }
}

async fn exec<P: AsRef<Path>>(cmd: &str, path: P) -> Result<Output> {
    #[cfg(target_os = "windows")]
    let (shell, first) = ("cmd", "/C");
    #[cfg(not(target_os = "windows"))]
    let (shell, first) = ("sh", "-c");
    Ok(Command::new(shell)
        .args(&[first, cmd])
        .current_dir(path)
        .output()
        .await?)
}

fn path(repo: &Repo, output_directory: &str) -> String {
    return format!(
        "{}/{}/{}",
        output_directory,
        repo.project_key.clone(),
        repo.name.clone()
    );
}

fn dir_exists(repo: &Repo, output_directory: &str) -> bool {
    Path::new(&path(repo, output_directory)).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "windows"))]
    const RM_STR: &str = "rm -rf test_repo";
    #[cfg(target_os = "windows")]
    const RM_STR: &str = "rmdir /Q /S test_repo";

    fn repo(project_key: &str, name: &str) -> Repo {
        Repo {
            project_key: String::from(project_key),
            git: String::from("https://github.com/jensim/bitbucket_server_cli.git"),
            name: String::from(name),
        }
    }

    #[tokio::test]
    async fn test_git_clone_and_update() {
        let repo_path = "/tmp/test_project/test_repo";
        let repo = repo("test_project", "test_repo");
        let project_path = "/tmp/test_project";
        let output_directory = "/tmp";
        std::fs::create_dir_all(project_path).unwrap();
        assert!(
            Path::new(project_path).exists(),
            "Project dir should exist."
        );
        if let Err(e) = exec(RM_STR, project_path).await {
            eprintln!("Failed removing {} due to {:?}", repo_path, e)
        }
        assert!(!Path::new(repo_path).exists(), "Repo dir should not exist.");

        git_clone(&repo, output_directory).await.unwrap();
        assert!(Path::new(repo_path).exists(), "Failed cleaning away dir.");

        git_update(&repo, output_directory).await.unwrap();
        assert!(Path::new(repo_path).exists(), "Failed cleaning away dir.");
    }
}
