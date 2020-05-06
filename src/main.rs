use bitbucket_server_cli::{cloner::Cloner, completion::gen_completions, types::Opts};
use generic_error::{GenericError, Result};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    match act().await {
        Ok(_) => Ok(()),
        Err(GenericError { msg }) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    }
}

async fn act() -> Result<()> {
    let opts: Opts = Opts::from_args();
    match opts {
        Opts::Clone(c) => Cloner::new(c)?.clone_projects().await,
        Opts::CloneUsers(c) => Cloner::new(c)?.clone_users().await,
        Opts::Completions => gen_completions(),
    }
}
