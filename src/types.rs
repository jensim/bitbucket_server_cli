use clap::arg_enum;
use generic_error::{GenericError, Result};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "BitBucket Server Cli")]
pub struct Opts {
    #[structopt(
        short = "I",
        long = "interactive",
        name = "interactive",
        help = "Run terminal in interactive mode, asking for required params like password user, host etc"
    )]
    pub interactive: bool,
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
        long = "concurrent_http",
        name = "bitbucket_concurrency",
        help = "Number of concurrent http requests towards bitbucket. Keep it sane, keep bitbucket alive for all. Max=100",
        default_value = "10"
    )]
    pub concurrency: usize,
    #[structopt(
        short = "H",
        long = "http_verbose",
        name = "bitbucket_verbose",
        help = "Output full http response on failed bitbucket requests."
    )]
    pub verbose: bool,
    #[structopt(
        short = "W",
        long = "env_password",
        name = "bitbucket_password_from_env",
        help = "Try get password from env variable BITBUCKET_PASSWORD.\nTry it out without showing your password:\nIFS= read -rs BITBUCKET_PASSWORD < /dev/tty  && export BITBUCKET_PASSWORD\n"
    )]
    pub password_from_env: bool,
    #[structopt(long = "clone_type",
        name = "clone_type",
        possible_values = & CloneType::variants(),
        case_insensitive = true,
        default_value = "ssh"
    )]
    pub clone_type: CloneType,
}

#[derive(StructOpt, Clone, Debug)]
pub struct GitOpts {
    #[structopt(
        short = "A",
        long = "all",
        name = "git_clone_all",
        help = "Clone all projects"
    )]
    pub clone_all: bool,
    #[structopt(
        short = "k",
        long = "key",
        name = "git_project_keys",
        help = "BitBucket Project keys"
    )]
    pub project_keys: Vec<String>,
    #[structopt(
        short = "R",
        long = "reset",
        name = "git_reset_state",
        help = "Reset repos before updating, \
        and switch to master branch"
    )]
    pub reset_state: bool,
    #[structopt(
        short = "g",
        long = "concurrent_git",
        name = "git_concurrency",
        help = "Number of concurrent git actions. Bitbucket might have a limited number of threads reserved for serving git requests - if you drive this value to high you might block your CI, colleagues or even crash bitbucket. Max=100",
        default_value = "3"
    )]
    pub concurrency: usize,
    #[structopt(
        short = "Q",
        long = "git_quiet",
        name = "git_quiet",
        help = "Suppress warnings from failed git actions."
    )]
    pub quiet: bool,
    #[structopt(
        long = "output_directory",
        help = "Suppress warnings from failed git actions.",
        default_value = "."
    )]
    pub output_directory: String,
}
arg_enum! {
    #[derive(Clone, Debug)]
    pub enum CloneType {
        SSH,
        HTTP
    }
}

#[derive(Deserialize, Debug)]
pub struct AllProjects {
    pub values: Vec<ProjDesc>,
}

#[derive(Deserialize, Debug)]
pub struct Projects {
    pub values: Vec<Project>,
}

impl Projects {
    pub fn get_clone_links(&self, opts: &BitBucketOpts) -> Vec<Repo> {
        let mut links: Vec<Repo> = Vec::new();
        let clone_type: &str = match opts.clone_type {
            CloneType::HTTP => "http",
            CloneType::SSH => "ssh",
        };
        for value in &self.values {
            for clone_link in &value.links.clone {
                if value.state.trim() == "AVAILABLE"
                    && value.scm_id.trim() == "git"
                    && clone_link.name.trim() == clone_type
                {
                    let git = if clone_type == "http"
                        && opts.password.is_some()
                        && opts.username.is_some()
                    {
                        if let Ok(a) = add_user_to_url(
                            &clone_link.href,
                            opts.username.clone().unwrap(),
                            opts.password.clone().unwrap(),
                        ) {
                            a
                        } else {
                            continue;
                        }
                    } else {
                        clone_link.href.clone()
                    };
                    links.push(Repo {
                        project_key: value.project.key.to_lowercase(),
                        name: value.slug.to_lowercase(),
                        git,
                    });
                }
            }
        }
        links
    }
}

fn add_user_to_url(url: &str, user: String, pass: String) -> Result<String> {
    if url.contains("://") {
        let mut url_parts: Vec<String> =
            url.split("://").map(move |s: &str| s.to_owned()).collect();
        url_parts.insert(1, "://".to_owned());
        url_parts.insert(2, user);
        url_parts.insert(3, ":".to_owned());
        url_parts.insert(4, pass);
        url_parts.insert(5, "@".to_owned());
        Ok(url_parts.join(""))
    } else {
        Err(GenericError {
            msg: format!("URL {} didn't contain '://'", url),
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub slug: String,
    pub scm_id: String,
    pub state: String,
    pub links: Links,
    pub project: ProjDesc,
}

#[derive(Deserialize, Debug)]
pub struct ProjDesc {
    pub key: String,
}

#[derive(Deserialize, Debug)]
pub struct Links {
    pub clone: Vec<CloneLink>,
}

#[derive(Deserialize, Debug)]
pub struct CloneLink {
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone)]
pub struct Repo {
    pub project_key: String,
    pub git: String,
    pub name: String,
}

#[derive(Debug)]
pub struct GitResult {
    pub project_key: String,
    pub success: String,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_user_to_url() {
        let added = add_user_to_url(
            &"http://localhost:7990/something.git".to_owned(),
            "admin".to_owned(),
            "password123".to_owned(),
        )
        .unwrap();
        assert_eq!(
            "http://admin:password123@localhost:7990/something.git",
            added.as_str(),
            "Url '{}' didn't match expectation.",
            added,
        )
    }

    #[test]
    fn test_http_with_user_and_password() {
        let repo_str = "https://localhost:7990/repo.git";
        let prjs = from(repo_str, CloneType::HTTP);
        let opts = BitBucketOpts {
            server: None,
            username: Some("admin".to_owned()),
            password: Some("password123".to_owned()),
            concurrency: 0,
            verbose: false,
            password_from_env: false,
            clone_type: CloneType::HTTP,
        };
        let vec1 = prjs.get_clone_links(&opts);
        assert_eq!(vec1.len(), 1);
        assert_eq!(
            vec1[0].git,
            "https://admin:password123@localhost:7990/repo.git"
        );
    }

    #[test]
    fn test_http_without_user_and_password() {
        let repo_str = "https://localhost:7990/repo.git";
        let prjs = from(repo_str, CloneType::HTTP);
        let opts = BitBucketOpts {
            server: None,
            username: None,
            password: None,
            concurrency: 0,
            verbose: false,
            password_from_env: false,
            clone_type: CloneType::HTTP,
        };
        let vec1 = prjs.get_clone_links(&opts);
        assert_eq!(vec1.len(), 1);
        assert_eq!(vec1[0].git, repo_str);
    }

    fn from(repo_str: &str, clone_type: CloneType) -> Projects {
        let prj = Project {
            slug: "asdf".to_string(),
            scm_id: "git".to_string(),
            state: "AVAILABLE".to_string(),
            links: Links {
                clone: vec![CloneLink {
                    name: format!("{}", clone_type).to_lowercase(),
                    href: repo_str.to_owned(),
                }],
            },
            project: ProjDesc { key: String::new() },
        };
        Projects { values: vec![prj] }
    }
}
