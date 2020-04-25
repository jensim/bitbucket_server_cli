use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "BitBucket Server Cli")]
pub struct Opts {
    #[structopt(short = "I", long = "interactive", name = "Run terminal in interactive mode, asking for required params like password user, host etc")]
    pub interactive: bool,
    #[structopt(flatten)]
    pub bitbucket_opts: BitBucketOpts,
    #[structopt(flatten)]
    pub git_opts: GitOpts,
}

#[derive(StructOpt, Clone, Debug)]
pub struct BitBucketOpts {
    #[structopt(short = "s", long = "server", name = "BitBucket server base url, http://example.bitbucket.mycompany.com")]
    pub server: Option<String>,
    #[structopt(short = "u", long = "username", name = "BitBucket username", )]
    pub username: Option<String>,
    #[structopt(short = "w", long = "password", name = "BitBucket password")]
    pub password: Option<String>,
    #[structopt(short = "b", long = "concurrent_http", name = "Number of concurrent http requests towards bitbucket. Keep it sane, keep bitbucket alive for all. Max=100", default_value = "10")]
    pub concurrency: usize,
}

#[derive(StructOpt, Clone, Debug)]
pub struct GitOpts {
    #[structopt(short = "A", long = "all", name = "Clone all projects")]
    pub clone_all: bool,
    #[structopt(short = "k", long = "key", name = "BitBucket Project keys")]
    pub project_keys: Vec<String>,
    #[structopt(short = "R", long = "reset", name = "Reset repos before updating, and switch to master branch")]
    pub reset_state: bool,
    #[structopt(short = "g", long = "concurrent_git", name = "Number of concurrent git actions. Bitbucket might have a limited number of threads reserved for serving git requests - if you drive this value to high you might block your CI, colleagues or even crash bitbucket. Max=100", default_value = "3")]
    pub concurrency: usize,
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
    pub fn get_clone_links(&self) -> Vec<Repo> {
        let mut links: Vec<Repo> = Vec::new();
        for value in &self.values {
            for clone_link in &value.links.clone {
                if value.state.trim() == "AVAILABLE" && value.scm_id.trim() == "git" && clone_link.name.trim() == "ssh" {
                    links.push(Repo {
                        project_key: value.project.key.to_lowercase(),
                        git: clone_link.href.clone(),
                        name: value.slug.to_lowercase()
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
