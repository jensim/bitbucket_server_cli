use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "BitBucket Server Cli")]
pub struct Opts {

    #[structopt(short = "A", long = "bit_bucket_project_all", name = "Check out all projects")]
    pub bit_bucket_project_all: bool,

    #[structopt(short = "k", long, name = "BitBucket Project key")]
    pub bit_bucket_project_key: Option<String>,

    #[structopt(short = "s", long = "server", name = "BitBucket server base url, http://example.bitbucket.mycompany.com")]
    pub bit_bucket_server: String,

    #[structopt(short = "p", long, name = "SSH private key password to auth against BitBucket git repo")]
    pub git_ssh_password: Option<String>,
    #[structopt(short = "u", long, name = "BitBucket user name")]
    pub bit_bucket_username: Option<String>,
    #[structopt(short = "w", long, name = "BitBucket password")]
    pub bit_bucket_password: Option<String>,
    #[structopt(short = "t", long, name = "Number of system threads", default_value = "3")]
    pub thread_count: usize,
    #[structopt(short = "R", long)]
    pub skip_reset_on_state: bool,
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

#[derive(Debug)]
pub struct Repo {
    pub project_key: String,
    pub git: String,
    pub name: String,
}
