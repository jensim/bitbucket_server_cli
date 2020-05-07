use futures::stream::{self, StreamExt};
use generic_error::Result;
use indicatif::{ProgressBar, ProgressStyle};

use crate::bitbucket::types::Repo;
use crate::git::single::SingleGit;
use crate::types::GitOpts;

mod single;

#[derive(Clone)]
pub struct Git<'a, 'b> {
    repos: &'a [Repo],
    opts: &'b GitOpts,
}

impl Git<'_, '_> {
    pub fn new<'a, 'b>(repos: &'a [Repo], opts: &'b GitOpts) -> Git<'a, 'b> {
        Git { repos, opts }
    }

    pub async fn git_going(self) {
        if self.repos.is_empty() {
            eprintln!("No repos to work on");
            return;
        }
        let progress_bar: ProgressBar = ProgressBar::new(self.repos.len() as u64);
        let bar_style = "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} (eta:{eta})";
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!("Working repos {}", bar_style))
                .progress_chars("#>-"),
        );
        let mut projects: Vec<&String> = self.repos.iter().map(|r| &r.project_key).collect();
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
            let opts_ref = &self.opts;
            let progress_bar = progress_bar.clone();
            async move {
                let git = SingleGit::new(repo, opts_ref);
                let result = git.clone_or_update().await;
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
