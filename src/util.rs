use std::path::Path;
use std::process::Output;

use generic_error::{GenericError, Result};
use tokio::process::Command as TokioCommand;

pub fn bail<T>(msg: &str) -> Result<T> {
    Err(GenericError {
        msg: msg.to_owned(),
    })
}

pub async fn exec<P: AsRef<Path>>(cmd: &str, path: P) -> Result<Output> {
    #[cfg(target_os = "windows")]
    let (shell, first) = ("cmd", "/C");
    #[cfg(not(target_os = "windows"))]
    let (shell, first) = ("sh", "-c");
    Ok(TokioCommand::new(shell)
        .args(&[first, cmd])
        .current_dir(path)
        .output()
        .await?)
}
