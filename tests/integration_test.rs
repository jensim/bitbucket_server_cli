#[cfg(test)]
mod tests {
    use bitbucket_server_cli::{
        cloner::Cloner,
        types::{BitBucketOpts, GitOpts, Opts},
    };

    #[tokio::test]
    #[ignore]
    async fn test() {
        let opts: Opts = Opts {
            interactive: false,
            bitbucket_opts: BitBucketOpts {
                server: Some("".to_owned()),
                username: None,
                password: None,
                concurrency: 5,
                verbose: true,
                password_from_env: false,
            },
            git_opts: GitOpts {
                clone_all: true,
                project_keys: vec![],
                reset_state: false,
                concurrency: 5,
                quiet: false,
            },
        };
        match Cloner::new(opts).git_clone().await {
            Ok(_) => {}
            Err(e) => {
                assert!(false, "Failed cloning {}", e.msg);
            }
        };
    }
}
