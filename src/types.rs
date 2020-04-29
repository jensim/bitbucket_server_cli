use clap::arg_enum;
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
    pub fn get_clone_links(&self, clone_type: CloneType) -> Vec<Repo> {
        let mut links: Vec<Repo> = Vec::new();
        let clone_type: &str = match clone_type {
            CloneType::HTTP => "http",
            CloneType::SSH => "ssh",
        };
        for value in &self.values {
            for clone_link in &value.links.clone {
                if value.state.trim() == "AVAILABLE"
                    && value.scm_id.trim() == "git"
                    && clone_link.name.trim() == clone_type
                {
                    links.push(Repo {
                        project_key: value.project.key.to_lowercase(),
                        git: clone_link.href.clone(),
                        name: value.slug.to_lowercase(),
                    });
                }
            }
        }
        return links;
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
