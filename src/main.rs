use bitbucket_server_cli::{cloner::Cloner, completion::gen_completions, types::Opts};
use generic_error::{GenericError, Result};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::from_args();
    let result: Result<()> = match opts {
        Opts::Clone(c) => Cloner::new(c).git_clone().await,
        Opts::Completions => gen_completions(),
    };
    match result {
        Ok(_) => Ok(()),
        Err(GenericError { msg }) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    }
}
