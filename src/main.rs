use structopt::StructOpt;

use bitbucket_server_cli::{
    cloner::Cloner,
    types::Opts,
};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::from_args();
    match Cloner::new(opts).git_clone().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e.msg);
            std::process::exit(1);
        }
    }
}
