use generic_error::Result;

use crate::types::{BitBucketOpts, CloneType};
use crate::util::bail;

pub fn get_clone_links(projects: Vec<Project>, opts: &BitBucketOpts) -> Vec<Repo> {
    let mut links: Vec<Repo> = Vec::new();
    let clone_type: &str = match opts.clone_type {
        CloneType::HTTP => "http",
        CloneType::HttpSavedLogin => "http",
        CloneType::SSH => "ssh",
    };
    for value in projects {
        for clone_link in &value.links.clone {
            if value.state.trim() == "AVAILABLE"
                && value.scm_id.trim() == "git"
                && clone_link.name.trim() == clone_type
            {
                let mut git = clone_link.href.clone();
                if let (CloneType::HttpSavedLogin, Some(user), Some(pass)) =
                    (&opts.clone_type, &opts.username, &opts.password)
                {
                    if let Ok(git_with_user) = add_user_to_url(&git, user, pass) {
                        git = git_with_user;
                    }
                }
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

fn add_user_to_url(url: &str, user: &str, pass: &str) -> Result<String> {
    if url.contains("://") {
        let mut url_parts: Vec<&str> = url
            .split("://")
            .map(move |s: &str| remove_possible_user_part(s))
            .collect();
        url_parts.insert(1, "://");
        url_parts.insert(2, user);
        url_parts.insert(3, ":");
        url_parts.insert(4, pass);
        url_parts.insert(5, "@");
        Ok(url_parts.join(""))
    } else {
        bail(&format!("URL {} didn't contain '://'", url))
    }
}

fn remove_possible_user_part(part: &str) -> &str {
    match part.find('@') {
        Some(i) => &part[i + 1..],
        None => part,
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

pub trait RepoUrlBuilder: std::fmt::Debug {
    fn get_repos_path(&self) -> String;
    fn get_filter_key(&self) -> String;
}

impl RepoUrlBuilder for ProjDesc {
    fn get_repos_path(&self) -> String {
        format!("/rest/api/latest/projects/{}/repos", &self.key)
    }

    fn get_filter_key(&self) -> String {
        self.key.to_lowercase()
    }
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

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct PageResponse<T> {
    pub is_last_page: bool,
    pub size: u32,
    pub limit: u32,
    pub values: Vec<T>,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct UserResult {
    pub slug: String,
    pub active: bool,
    pub name: String,
    pub display_name: String,
}

impl RepoUrlBuilder for UserResult {
    fn get_repos_path(&self) -> String {
        format!("/rest/api/1.0/users/{}/repos", self.slug)
    }

    fn get_filter_key(&self) -> String {
        format!("~{}", self.slug.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_user_to_url_with_user() {
        let added = add_user_to_url(
            &"http://user@localhost:7990/something.git".to_owned(),
            "admin",
            "password123",
        )
        .unwrap();
        assert_eq!(
            "http://admin:password123@localhost:7990/something.git", &added,
            "Url '{}' didn't match expectation.",
            &added,
        )
    }

    #[test]
    fn test_add_user_to_url() {
        let added = add_user_to_url(
            &"http://localhost:7990/something.git".to_owned(),
            "admin",
            "password123",
        )
        .unwrap_or("error".to_owned());

        assert_eq!(
            "http://admin:password123@localhost:7990/something.git", &added,
            "Url '{}' didn't match expectation.",
            added,
        )
    }

    #[test]
    fn test_http_with_user_and_password() {
        let repo_str = "https://localhost:7990/repo.git";
        let prjs = from(repo_str, CloneType::HttpSavedLogin);
        let opts = BitBucketOpts {
            server: None,
            username: Some("admin".to_owned()),
            password: Some("password123".to_owned()),
            concurrency: 0,
            verbose: false,
            password_from_env: false,
            clone_type: CloneType::HttpSavedLogin,
            project_keys: vec!["key".to_owned()],
            all: false,
        };
        let vec1 = get_clone_links(prjs, &opts);
        assert_eq!(vec1.len(), 1, "Wrong number of output Repo objects");
        assert_eq!(
            vec1[0].git,
            "https://admin:password123@localhost:7990/repo.git"
        );
    }

    #[test]
    fn test_http_without_user_and_password() {
        let repo_str = "https://admin@localhost:7990/repo.git";
        let prjs = from(repo_str, CloneType::HTTP);
        let opts = BitBucketOpts {
            server: None,
            username: Some("admin".to_owned()),
            password: None,
            concurrency: 0,
            verbose: false,
            password_from_env: false,
            clone_type: CloneType::HTTP,
            project_keys: vec!["key".to_owned()],
            all: false,
        };
        let vec1 = get_clone_links(prjs, &opts);
        assert_eq!(vec1.len(), 1);
        assert_eq!(vec1[0].git, repo_str);
    }

    fn from(repo_str: &str, clone_type: CloneType) -> Vec<Project> {
        let clone_type = match clone_type {
            CloneType::SSH => "ssh",
            CloneType::HTTP => "http",
            CloneType::HttpSavedLogin => "http",
        };
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
        vec![prj]
    }
}
