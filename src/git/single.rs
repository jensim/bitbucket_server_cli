use crate::bitbucket::types::Repo;
use crate::types::GitOpts;
use crate::util::{bail, exec};
use generic_error::Result;
use std::path::Path;

#[derive(Clone)]
pub struct SingleGit<'a, 'b> {
    repo: &'a Repo,
    opts: &'b GitOpts,
}

impl SingleGit<'_, '_> {
    pub fn new<'a, 'b>(repo: &'a Repo, opts: &'b GitOpts) -> SingleGit<'a, 'b> {
        SingleGit { repo, opts }
    }

    pub async fn clone_or_update(&self) -> Result<()> {
        if self.dir_exists() {
            if self.opts.reset_state {
                self.git_reset().await?;
            }
            self.git_update().await?;
        } else {
            self.git_clone().await?;
        }
        Ok(())
    }

    async fn git_clone(&self) -> Result<()> {
        let string_path = format!("{}/{}", self.opts.output_directory, self.repo.project_key);
        let path = Path::new(&string_path);

        let fail_suffix = format!("failed git clone into {}", self.opts.output_directory);
        match exec(
            &*format!("git clone {} {}", self.repo.git, self.repo.name),
            path,
        )
        .await
        {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) if !o.status.success() => {
                self.generate_repo_err_from_output(&fail_suffix, o.stdout, o.stderr)
            }
            Err(e) => self.generate_repo_err(&fail_suffix, &e.msg),
            _ => self.generate_repo_err(&fail_suffix, "unknown"),
        }
    }

    async fn git_update(&self) -> Result<()> {
        let string_path = self.path();
        let path = Path::new(&string_path);

        let fail_suffix = "failed git pull";
        match exec("git pull --autostash --ff-only --rebase", path).await {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) if !o.status.success() => {
                self.generate_repo_err_from_output(fail_suffix, o.stdout, o.stderr)
            }
            Err(e) => self.generate_repo_err(fail_suffix, &e.msg),
            _ => self.generate_repo_err(fail_suffix, "unknown"),
        }
    }

    async fn git_reset(&self) -> Result<()> {
        let string_path = self.path();
        let path = Path::new(&string_path);
        match exec("git reset --hard", path).await {
            Ok(_) => match exec("git checkout main --quiet --force", path).await {
                Err(e) => self.generate_repo_err("failed 'checkout main'", &e.msg),
                Ok(_) => Ok(()),
            },
            Err(e) => self.generate_repo_err("failed resetting repo", &e.msg),
        }
    }

    fn generate_repo_err_from_output<T>(
        &self,
        suffix: &str,
        cause_out: Vec<u8>,
        cause_err: Vec<u8>,
    ) -> Result<T> {
        let cause = match (cause_to_str(cause_err), cause_to_str(cause_out)) {
            (Some(e), Some(o)) => format!("Err: '{}' Output: '{}'", e.trim(), o.trim()),
            (Some(e), None) => format!("Err: '{}'", e.trim()),
            (None, Some(o)) => format!("Output: '{}'", o.trim()),
            (None, None) => "no output".to_string(),
        };
        self.generate_repo_err(suffix, &cause)
    }

    fn path(&self) -> String {
        return format!(
            "{}/{}/{}",
            &self.opts.output_directory, &self.repo.project_key, &self.repo.name
        );
    }

    fn dir_exists(&self) -> bool {
        Path::new(&self.path()).exists()
    }

    fn generate_repo_err<T>(&self, suffix: &str, cause: &str) -> Result<T> {
        bail(&format!(
            "{}/{} {}. Cause: {}",
            &self.repo.project_key, self.repo.name, suffix, cause
        ))
    }
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
        let opts = GitOpts {
            reset_state: false,
            concurrency: 1,
            quiet: false,
            output_directory: output_directory.to_owned(),
        };
        std::fs::create_dir_all(project_path).unwrap();
        assert!(
            Path::new(project_path).exists(),
            "Project dir should exist."
        );
        if let Err(e) = exec(RM_STR, project_path).await {
            eprintln!("Failed removing {} due to {:?}", repo_path, e)
        }
        assert!(!Path::new(repo_path).exists(), "Repo dir should not exist.");
        let single = SingleGit::new(&repo, &opts);

        single.git_clone().await.unwrap();
        assert!(Path::new(repo_path).exists(), "Failed cleaning away dir.");

        single.git_update().await.unwrap();
        assert!(Path::new(repo_path).exists(), "Failed cleaning away dir.");
    }
}
