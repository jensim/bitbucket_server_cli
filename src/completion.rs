use std::path::Path;

use clap::Shell;
use dialoguer::{Confirm, Input, Select};
use generic_error::Result;
use structopt::StructOpt;

use crate::types::Opts;
use crate::util::bail;

#[cfg(target_os = "windows")]
const HOME_VAR: &str = "USERPROFILE";
#[cfg(not(target_os = "windows"))]
const HOME_VAR: &str = "HOME";

pub fn gen_completions() -> Result<()> {
    let mut app: clap::App = Opts::clap();
    if confirm("Do you want to generate completions for bitbucket_server_cli?")? {
        let shell = get_shell_selection()?;
        let output_dir = get_output_dir(shell)?;
        if confirm(&format!(
            "Do you want to generate {} completions into {}",
            shell, output_dir
        ))? {
            app.gen_completions("bitbucket_server_cli", shell, output_dir);
        }
    }
    Ok(())
}

fn confirm(question: &str) -> Result<bool> {
    match Confirm::new()
        .default(false)
        .with_prompt(question)
        .interact()
    {
        Ok(ans) => Ok(ans),
        Err(e) => bail(&format!(
            "Failed getting answer for question due to {:?}",
            e
        )),
    }
}

fn create_dir_if_necessary(output_dir: &str) -> Result<()> {
    if !Path::new(output_dir).exists() {
        match Confirm::new()
            .with_prompt(format!(
                "Output directory does not exist. Do you want to create '{}' ?",
                output_dir
            ))
            .default(false)
            .interact()
        {
            Ok(ans) if ans => match std::fs::create_dir_all(output_dir) {
                Ok(_) => {
                    println!("Directory created.");
                    Ok(())
                }
                Err(e) => bail(&format!(
                    "Failed creating dir {} due to {:?}",
                    output_dir, e
                )),
            },
            Ok(_ans) => {
                bail("Cannot proceed writing to output_directory without creating it first.")
            }
            Err(e) => bail(&format!("Failed reading input due to {:?}", e)),
        }
    } else {
        Ok(())
    }
}

fn get_output_dir(shell: Shell) -> Result<String> {
    let mut input = Input::new();
    if let Some(output_dir_default) = get_default_completion_location(shell)? {
        input.default(output_dir_default);
    }
    match input.with_prompt("Output directory").interact() {
        Ok(output_dir) => {
            create_dir_if_necessary(&output_dir)?;
            Ok(output_dir)
        }
        Err(e) => bail(&format!("Failed interpreting prompt due to {:?}", e)),
    }
}

fn get_shell_selection() -> Result<Shell> {
    let variants1 = clap::Shell::variants();
    match Select::new()
        .with_prompt("Shell")
        .default(0)
        .items(&variants1)
        .interact()
    {
        Ok(shell_idx) => {
            let shell_str = variants1[shell_idx];
            match shell_str.parse() {
                Ok(shell) => Ok(shell),
                Err(e) => bail(&format!("Failed parsing shell selection due to {}", e)),
            }
        }
        Err(e) => bail(&format!("Failed determining selection due to {:?}", e)),
    }
}

fn get_default_completion_location(shell: Shell) -> Result<Option<String>> {
    let home = std::env::var(HOME_VAR);
    match (shell, home) {
        (Shell::Zsh, Ok(home)) => Ok(join_path(&[&home, ".oh-my-zsh", "completions"])),
        (Shell::Fish, Ok(home)) => Ok(join_path(&[&home, ".config", "fish", "completions"])),
        (Shell::Bash, _) => Ok(join_path(&["/usr", "local", "share", "bash-completion"])),
        (Shell::PowerShell, Ok(home)) => Ok(join_path(&[&home, "Documents", "WindowsPowerShell"])),
        (_, Err(e)) => {
            eprintln!("Failed reading {} variable due to {:?}", HOME_VAR, e);
            Ok(None)
        }
        (_, Ok(_)) => Ok(None),
    }
}

fn join_path(parts: &[&str]) -> Option<String> {
    let mut path = Path::new(parts[0]).to_path_buf();
    for part in parts[1..].to_vec() {
        path.push(part);
    }
    path.to_str().map(|s| s.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_home() {
        let home = std::env::var(HOME_VAR).unwrap();
        #[cfg(target_os = "windows")]
        assert!(home.to_lowercase().starts_with(r"c:\"));
        #[cfg(not(target_os = "windows"))]
        assert!(home.starts_with("/"));
    }

    #[test]
    fn test_join_path() {
        let home = std::env::var(HOME_VAR).unwrap();
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(join_path(&[&home, "foo"]).unwrap(), format!("{}/foo", home))
        }
        #[cfg(target_os = "windows")]
        {
            assert_eq!(
                join_path(&[&home, "foo"]).unwrap(),
                format!(r"{}\foo", home)
            )
        }
    }

    #[test]
    fn test_shell_default_location() {
        let home = std::env::var(HOME_VAR).unwrap();
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(
                get_default_completion_location(Shell::Zsh)
                    .unwrap()
                    .unwrap(),
                format!("{}/.oh-my-zsh/completions", home)
            );
        }
        #[cfg(target_os = "windows")]
        {
            assert_eq!(
                get_default_completion_location(Shell::PowerShell)
                    .unwrap()
                    .unwrap(),
                format!(r"{}\Documents\WindowsPowerShell", home)
            );
        }
    }
}
