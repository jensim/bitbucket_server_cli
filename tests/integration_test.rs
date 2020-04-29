extern crate bitbucket_server_cli;

use generic_error::Result;

use bitbucket_server_cli::{
    cloner::Cloner,
    types::{BitBucketOpts, CloneType, GitOpts, Opts},
};

fn env(key: &str) -> Result<String> {
    match std::env::var(key) {
        Ok(r) => Ok(r),
        _ => panic!(format!("{} was not set.", key)),
    }
}

fn opts() -> Result<Opts> {
    Ok(Opts {
        interactive: false,
        bitbucket_opts: BitBucketOpts {
            server: Some(env("BITBUCKET_SERVER")?),
            username: Some(env("BITBUCKET_USER")?),
            password: Some(env("BITBUCKET_PASSWORD")?),
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
        },
    })
}

#[tokio::test]
async fn test_ssh() {
    let opts: Opts = opts().unwrap();
    let bitbucket_project = env("BITBUCKET_PROJECT").unwrap();
    match Cloner::new(opts).git_clone().await {
        Ok(_) => {}
        Err(e) => {
            assert!(false, "Failed cloning {}", e.msg);
        }
    };
    let dir = std::fs::read_dir(&bitbucket_project).unwrap();
    let mut found_git_dir = false;
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
    assert!(found_git_dir, "No git dirs found. I am disappointed.");
}

#[tokio::test]
async fn test_http() {
    let opts: Opts = opts().unwrap();
    match Cloner::new(opts).git_clone().await {
        Ok(_) => {}
        Err(e) => {
            assert!(false, "Failed cloning {}", e.msg);
        }
    };
}
