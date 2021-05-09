use std::path::Path;

use clap::arg_enum;
use generic_error::Result;
use structopt::StructOpt;

use crate::input::prompts::{PROMPT_BB_PROJECT_ALL, PROMPT_BB_SERVER, PROMPT_BB_USERNAME};
use crate::input::{get_bool, get_password, get_with_default, password_from_env};
use crate::util::bail;
use dialoguer::Confirm;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "BitBucket Server Cli",
    about = "Clone a thousand repos, and keep em up to date, no problem."
)]
pub enum Opts {
    #[structopt(about = "Clone projects and users combined")]
    Clone(CloneOpts),
    #[structopt(about = "Clone projects")]
    CloneProjects(CloneOpts),
    #[structopt(about = "Clone users")]
    CloneUsers(CloneOpts),
    #[structopt(about = "Generate shell completions")]
    Completions,
}

#[derive(StructOpt, Debug, Clone)]
pub struct CloneOpts {
    #[structopt(
        short = "B",
        long = "batch",
        name = "batch_mode",
        help = "Run terminal in batch mode, with no interactions."
    )]
    pub batch_mode: bool,
    #[structopt(flatten)]
    pub bitbucket_opts: BitBucketOpts,
    #[structopt(flatten)]
    pub git_opts: GitOpts,
}

#[derive(StructOpt, Clone, Debug)]
pub struct BitBucketOpts {
    #[structopt(
        short = "s",
        long = "server",
        name = "bitbucket_server",
        help = "BitBucket server base url, http://example.bitbucket.mycompany.com"
    )]
    pub server: Option<String>,
    #[structopt(
        short = "u",
        long = "username",
        name = "bitbucket_username",
        help = "BitBucket username"
    )]
    pub username: Option<String>,
    #[structopt(
        short = "w",
        long = "password",
        name = "bitbucket_password",
        help = "BitBucket password"
    )]
    pub password: Option<String>,
    #[structopt(
        short = "b",
        long = "concurrent-http",
        name = "bitbucket_concurrency",
        help = "Number of concurrent http requests towards bitbucket. Keep it sane, keep bitbucket alive for all. Max=100",
        default_value = "20"
    )]
    pub concurrency: u8,
    #[structopt(
        short = "H",
        long = "http-verbose",
        name = "bitbucket_verbose",
        help = "Output full http response on failed bitbucket requests."
    )]
    pub verbose: bool,
    #[structopt(
        short = "W",
        long = "env-password",
        name = "bitbucket_password_from_env",
        help = "Try get password from env variable BITBUCKET_PASSWORD.\nTry it out without showing your password:\nIFS= read -rs BITBUCKET_PASSWORD < /dev/tty  && export BITBUCKET_PASSWORD\n"
    )]
    pub password_from_env: bool,
    #[structopt(
        long = "clone-type",
        name = "clone_type",
        possible_values = & CloneType::variants(),
        case_insensitive = true,
        default_value = "ssh"
    )]
    pub clone_type: CloneType,
    #[structopt(
        short = "k",
        long = "key",
        name = "git_project_keys",
        help = "BitBucket Project keys (applicable multiple times)"
    )]
    pub project_keys: Vec<String>,
    #[structopt(
        short = "A",
        long = "all",
        name = "bitbucket_all",
        help = "Clone all projects"
    )]
    pub all: bool,
    #[structopt(
        long = "http-timeout",
        help = "HTTP timout, seconds.",
        default_value = "3"
    )]
    pub timeout_sec: u64,
    #[structopt(
        long,
        help = "Retries to attempt requesting on timeout from bitbucket.",
        default_value = "2"
    )]
    pub retries: u32,
    #[structopt(
        long = "http-backoff",
        help = "Linear backoff time per failed request, expressed in seconds.\nie. 10 timed out requests and backoff=10s -> 100s backoff on next timed out request"
    )]
    pub backoff_sec: Option<u64>,
}

#[derive(StructOpt, Clone, Debug)]
pub struct GitOpts {
    #[structopt(
        short = "R",
        long = "reset",
        name = "git_reset_state",
        help = "Reset repos before updating, \
        and switch to main branch"
    )]
    pub reset_state: bool,
    #[structopt(
        short = "g",
        long = "concurrent-git",
        name = "git_concurrency",
        help = "Number of concurrent git actions. Bitbucket might have a limited number of threads reserved for serving git requests - if you drive this value to high you might block your CI, colleagues or even crash bitbucket. Max=100",
        default_value = "5"
    )]
    pub concurrency: usize,
    #[structopt(
        short = "Q",
        long = "git-quiet",
        name = "git_quiet",
        help = "Suppress warnings from failed git actions."
    )]
    pub quiet: bool,
    #[structopt(
        long = "output-directory",
        help = "Suppress warnings from failed git actions.",
        default_value = "."
    )]
    pub output_directory: String,
}
arg_enum! {
    #[allow(clippy::upper_case_acronyms)]
    #[derive(Clone, Debug)]
    pub enum CloneType {
        SSH,
        HTTP,
        HttpSavedLogin,
    }
}

impl CloneOpts {
    pub fn validate(&mut self) -> Result<()> {
        if self.interactive() {
            self.bitbucket_opts.server = match self.bitbucket_opts.server.clone() {
                None => get_with_default(&PROMPT_BB_SERVER, None, false),
                Some(s) => Some(s),
            };
            self.bitbucket_opts.username = match self.bitbucket_opts.username.clone() {
                None => get_with_default(&PROMPT_BB_USERNAME, None, true),
                Some(s) => Some(s),
            };
            self.bitbucket_opts.password = match self.bitbucket_opts.username {
                None => None,
                Some(_) if self.bitbucket_opts.password_from_env => None,
                Some(_) if self.bitbucket_opts.password.is_none() => get_password(),
                _ => None,
            };
            self.bitbucket_opts.all = self.bitbucket_opts.all
                || (self.bitbucket_opts.project_keys().is_empty()
                    && get_bool(&PROMPT_BB_PROJECT_ALL, false));
        }
        self.do_create_output_dir()?;
        if self.bitbucket_opts.server.is_none() {
            bail("Server is required")?;
        } else if !self.bitbucket_opts.all
            && self.bitbucket_opts.project_keys().is_empty()
            && self.batch_mode
        {
            bail("project selection is required (all or keys)")?;
        } else if self.git_opts.concurrency > 100 || self.bitbucket_opts.concurrency > 100 {
            bail("Max concurrent actions = 100")?;
        } else if !Path::new(&self.git_opts.output_directory).exists() {
            bail("output_directory is not accessible, does it exist?")?;
        }
        if self.bitbucket_opts.password_from_env {
            match password_from_env() {
                Ok(pass) => self.bitbucket_opts.password = Some(pass),
                Err(e) => bail(&format!("Failed getting env password. {}", e.msg))?,
            }
        }
        Ok(())
    }

    fn do_create_output_dir(&self) -> Result<()> {
        if !Path::new(&self.git_opts.output_directory).exists() {
            if self.batch_mode {
                bail(&format!(
                    "Output directory {} doesn't exist",
                    &self.git_opts.output_directory
                ))?;
            }
            match Confirm::new()
                .with_prompt(&format!(
                    "Output dir {} does not exist, want me to create it?",
                    &self.git_opts.output_directory
                ))
                .default(false)
                .interact()
            {
                Ok(true) => match std::fs::create_dir_all(&self.git_opts.output_directory) {
                    Ok(_) => {}
                    _ => bail("Failed creating output directory.")?,
                },
                Ok(false) => bail("Unable to proceed without an output directory")?,
                Err(e) => bail(&format!("{:?}", e))?,
            }
        }
        Ok(())
    }

    pub fn interactive(&self) -> bool {
        !self.batch_mode
    }
}

impl BitBucketOpts {
    pub fn project_keys(&self) -> Vec<String> {
        self.project_keys
            .iter()
            .map(|key| key.to_lowercase())
            .collect()
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parsing() {
        let opt: Opts = Opts::from_iter(&[
            "bitbucket_server_cli",
            "clone-projects",
            "--http-timeout",
            "10",
        ]);
        match opt {
            Opts::CloneProjects(co) => {
                assert_eq!(
                    co.bitbucket_opts.timeout_sec, 10,
                    "http timeout was not set correctly"
                )
            }
            _ => panic!("Bad format"),
        }
    }
}
