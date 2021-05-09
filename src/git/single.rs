use crate::bitbucket::types::Repo;
use crate::types::GitOpts;
use crate::util::{bail, exec};
use generic_error::Result;
use std::path::Path;
use std::process::Output;

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

        self.exec_resolve(
            &format!("git clone into {}", self.opts.output_directory),
            &format!("git clone {} {}", self.repo.git, self.repo.name),
            path,
        )
        .await?;
        Ok(())
    }

    async fn git_update(&self) -> Result<()> {
        let string_path = self.path();
        let path = Path::new(&string_path);
        let current_branch_raw: Vec<u8> =
            exec("git rev-parse --abbrev-ref HEAD", path).await?.stdout;
        let current_branch: &str = std::str::from_utf8(&current_branch_raw)?.trim();
        let main_branch: String = self.get_git_main().await?;

        if main_branch == "(unknown)" {
            self.generate_repo_err("get main branch", "main branch unknown")?;
        } else if current_branch == main_branch.as_str() {
            self.exec_resolve("git pull", "git pull --autostash --ff-only --rebase", path)
                .await?;
        } else {
            self.exec_resolve(
                "git fetch",
                &format!("git fetch origin {}:{}", main_branch, main_branch),
                path,
            )
            .await?;
            match self
                .exec_resolve("git pull", "git pull --autostash --ff-only --rebase", path)
                .await
            {
                _ => {}
            }
            self.exec_resolve(
                &format!("checkout {}", main_branch),
                &format!("git checkout {}", main_branch),
                path,
            )
            .await?;
        }
        self.clean_merged_branches(&main_branch, path).await?;
        Ok(())
    }

    async fn clean_merged_branches(&self, main_branch: &str, path: &Path) -> Result<()> {
        let out_raw: Vec<u8> = self
            .exec_resolve("list merged branches", "git branch --merged", path)
            .await?
            .stdout;
        let out: &str = std::str::from_utf8(&out_raw)?;
        for line in out.lines() {
            if !line.starts_with("* ") && line != format!("  {}", main_branch) {
                self.exec_resolve(
                    "remove branch",
                    &format!("git branch -D {}", line.trim()),
                    path,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn exec_resolve(&self, action: &str, cmd: &str, path: &Path) -> Result<Output> {
        match exec(cmd, path).await {
            Ok(o) if o.status.success() => Ok(o),
            Ok(o) if !o.status.success() => {
                self.generate_repo_err_from_output(action, o.stdout, o.stderr)
            }
            Err(e) => self.generate_repo_err(action, &e.msg),
            _ => self.generate_repo_err(action, "unknown"),
        }
    }

    async fn get_git_main(&self) -> Result<String> {
        let path_string = self.path();
        let path = Path::new(&path_string);
        let raw: Vec<u8> = self
            .exec_resolve("git remote show origin", "git remote show origin", path)
            .await?
            .stdout;
        let stdout: &str = std::str::from_utf8(&raw)?;
        self.get_git_main_from_remote_info(stdout)
    }

    fn get_git_main_from_remote_info(&self, remote_info: &str) -> Result<String> {
        let opt: Option<String> = remote_info
            .lines()
            .find(|s| s.starts_with("  HEAD branch: "))
            .map(|s| String::from(&s[15..]));
        match opt {
            Some(s) => Ok(s),
            None => self.generate_repo_err("list branches", "unable to filter main branch"),
        }
    }

    async fn git_reset(&self) -> Result<()> {
        let string_path = self.path();
        let path = Path::new(&string_path);
        match exec("git reset --hard", path).await {
            Ok(_) => {
                let main_branch: String = self.get_git_main().await?;
                match exec(
                    &format!("git checkout {} --quiet --force", main_branch),
                    path,
                )
                .await
                {
                    Err(e) => self.generate_repo_err(&format!("checkout {}", main_branch), &e.msg),
                    Ok(_) => Ok(()),
                }
            }
            Err(e) => self.generate_repo_err("resetting repo", &e.msg),
        }
    }

    fn generate_repo_err_from_output<T>(
        &self,
        suffix: &str,
        cause_out: Vec<u8>,
        cause_err: Vec<u8>,
    ) -> Result<T> {
        let cause = match (cause_to_str(cause_err), cause_to_str(cause_out)) {
            (Some(e), _) => format!("Err: '{}'", e.trim()),
            (_, Some(o)) => format!("Output: '{}'", o.trim()),
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

    fn generate_repo_err<T>(&self, action: &str, cause: &str) -> Result<T> {
        bail(&format!(
            "{}/{} failed {}. Cause: {}",
            &self.repo.project_key,
            self.repo.name,
            action,
            cause.lines().next().unwrap_or("unknown")
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
        assert!(Path::new(repo_path).exists(), "Failed cloning");

        single.git_update().await.unwrap();

        if let Err(e) = exec(RM_STR, project_path).await {
            eprintln!("Failed removing {} due to {:?}", repo_path, e)
        }
        assert!(!Path::new(repo_path).exists(), "Failed cleaning away dir.");
    }

    #[test]
    fn test_get_git_main_from_remote_info() {
        //let repo_path = "/tmp/test_project/test_repo";
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

        let single = SingleGit::new(&repo, &opts);
        let s = single.get_git_main_from_remote_info(
            "* remote origin
  Fetch URL: ssh://git@github.com/jensim/foo-bar-baz.git
  Push  URL: ssh://git@github.com/jensim/foo-bar-baz.git
  HEAD branch: master
  Remote branches:
    master                                                       tracked
  Local branches configured for 'git pull':
    master                                                       merges with remote master
  Local refs configured for 'git push':
    master                                                       pushes to master                                                       (up to date)
",
        ).unwrap();

        assert_eq!(s, "master")
    }
}
