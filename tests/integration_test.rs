extern crate bitbucket_server_cli;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use generic_error::{GenericError, Result};

use bitbucket_server_cli::types::CloneOpts;
use bitbucket_server_cli::{
    cloner::Cloner,
    types::{BitBucketOpts, CloneType, GitOpts},
};

fn env(key: &str) -> Result<String> {
    match std::env::var(key) {
        Ok(r) => Ok(r),
        Err(_) => Err(GenericError {
            msg: format!("{} was not set.", key),
        }),
    }
}

fn env_option(key: &str) -> Option<String> {
    match env(key) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

fn random_string(len: usize) -> String {
    thread_rng().sample_iter(&Alphanumeric).take(len).collect()
}

fn opts() -> Result<CloneOpts> {
    Ok(CloneOpts {
        batch_mode: true,
        bitbucket_opts: BitBucketOpts {
            server: Some(env("BITBUCKET_SERVER")?),
            username: env_option("BITBUCKET_USER"),
            password: env_option("BITBUCKET_PASSWORD"),
            concurrency: 5,
            verbose: true,
            password_from_env: false,
            clone_type: CloneType::SSH,
        },
        git_opts: GitOpts {
            clone_all: false,
            project_keys: vec![env("BITBUCKET_PROJECT")?],
            reset_state: false,
            concurrency: 5,
            quiet: false,
            output_directory: format!("/tmp/test_clone_{}", random_string(12)),
        },
    })
}

#[tokio::test]
async fn test_ssh() {
    let opts: CloneOpts = opts().unwrap();
    let bitbucket_project = env("BITBUCKET_PROJECT").unwrap();
    let output_directory = opts.git_opts.output_directory.clone();
    assert!(
        std::fs::create_dir_all(&output_directory).is_ok(),
        "Failed creating output dir for test {}.",
        &output_directory
    );
    let path = format!("{}/{}", &output_directory, &bitbucket_project);
    match Cloner::new(opts).unwrap().clone_projects().await {
        Ok(_) => {}
        Err(e) => {
            assert!(false, "Failed cloning {}", e.msg);
        }
    };
    let mut found_git_dir = false;
    let dir = std::fs::read_dir(&path).unwrap();
    'outer: for dir in dir {
        if dir.is_ok() {
            let dir = dir.unwrap();
            let dir = std::fs::read_dir(dir.path()).unwrap();
            for dir in dir {
                if dir.is_ok() {
                    let dir = dir.unwrap();
                    if dir.file_name().to_str().unwrap().eq(".git") {
                        if std::fs::read_dir(dir.path()).is_ok() {
                            found_git_dir = true;
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    if let Err(e) = std::fs::remove_dir_all(&output_directory) {
        eprintln!("Failed removing {} due to {:?}", &path, e)
    }
    assert!(found_git_dir, "No git dirs found. I am disappointed.");
}

#[tokio::test]
async fn test_http() {
    let opts: CloneOpts = opts().unwrap();
    let output_directory = opts.git_opts.output_directory.clone();
    assert!(
        std::fs::create_dir_all(&output_directory).is_ok(),
        "Failed creating output dir for test {}.",
        &output_directory
    );
    let result = Cloner::new(opts).unwrap().clone_projects().await;
    if let Err(e) = std::fs::remove_dir_all(&output_directory) {
        eprintln!("Failed removing {} due to {:?}", &output_directory, e);
    }
    if let Err(e) = result {
        assert!(false, "Failed cloning {}", e.msg);
    }
}

#[tokio::test]
async fn test_user_http() {
    let mut opts: CloneOpts = opts().unwrap();
    opts.git_opts.project_keys = vec![env("BITBUCKET_USER").unwrap()];
    let output_directory = opts.git_opts.output_directory.clone();
    assert!(
        std::fs::create_dir_all(&output_directory).is_ok(),
        "Failed creating output dir for test {}.",
        &output_directory
    );
    let result = Cloner::new(opts).unwrap().clone_users().await;
    if let Err(e) = std::fs::remove_dir_all(&output_directory) {
        eprintln!("Failed removing {} due to {:?}", &output_directory, e);
    }
    if let Err(e) = result {
        assert!(false, "Failed cloning {}", e.msg);
    }
}
