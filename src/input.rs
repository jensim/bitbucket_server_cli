use std::result::Result as StdResult;

use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Password};
use generic_error::Result;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};

use crate::bitbucket::types::Repo;
use crate::prompts::{Prompt, PROMPT_BB_PROJECT_SOME};
use crate::util::bail;

const PROP_FILE: &str = ".bitbucket_server_cli.db";

pub fn select_projects(repos: &[Repo]) -> Vec<String> {
    let mut project_keys: Vec<String> = Vec::new();
    for r in repos {
        if !project_keys.contains(&r.project_key) {
            project_keys.push(r.project_key.clone());
        }
    }
    project_keys.sort();
    let mut db = get_db();
    let previous: Vec<String> = db
        .get(PROMPT_BB_PROJECT_SOME.db_key)
        .unwrap_or_else(Vec::new);
    let pre_selected: Vec<bool> = project_keys
        .iter()
        .map(|key| previous.contains(key))
        .collect();
    let mut answer: Vec<usize> = Vec::new();
    while answer.is_empty() {
        answer = MultiSelect::new()
            .items(&project_keys)
            .with_prompt(PROMPT_BB_PROJECT_SOME.prompt_str)
            .defaults(&pre_selected[..])
            .interact()
            .unwrap_or_else(|_e| {
                eprintln!("Failed handling project key selection");
                std::process::exit(1);
            });
    }
    let mut filtered: Vec<String> = Vec::new();
    for i in answer {
        filtered.push(project_keys[i].clone());
    }
    if db.set(PROMPT_BB_PROJECT_SOME.db_key, &filtered).is_err() {
        eprintln!(
            "Failed writing value {} to prickle db",
            PROMPT_BB_PROJECT_SOME.db_key
        );
    };
    filtered
}

pub fn password_from_env() -> Result<String> {
    let key = "BITBUCKET_PASSWORD";
    match std::env::var(key) {
        Ok(val) => Ok(val),
        Err(e) => bail(&format!(
            "{} is not defined in the environment. {:?}",
            key, e
        )),
    }
}

pub fn get_db() -> PickleDb {
    PickleDb::load(
        PROP_FILE,
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Yaml,
    )
    .unwrap_or_else(|_| {
        PickleDb::new(
            PROP_FILE,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Yaml,
        )
    })
}

pub fn get_password(prompt: &Prompt) -> Option<String> {
    let password = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.prompt_str)
        .allow_empty_password(true)
        .interact();
    resolve(password)
}

pub fn get_bool(prompt: &Prompt, default: bool) -> bool {
    let mut db = get_db();
    let read_val: bool = db.get(prompt.db_key).unwrap_or(default);
    let prompt_val = Confirm::new()
        .with_prompt(prompt.prompt_str)
        .default(read_val)
        .show_default(true)
        .interact()
        .unwrap_or(default);
    if db.set(prompt.db_key, &prompt_val).is_err() {
        eprintln!("Failed writing value {} to prickle db", prompt.db_key)
    };
    prompt_val
}

pub fn get_with_default(
    prompt: &Prompt,
    default: Option<String>,
    allow_empty: bool,
) -> Option<String> {
    let mut db = get_db();
    let read_val: Option<String> = db.get(prompt.db_key).or(default);
    let ask: Option<String> = match read_val {
        Some(s) => resolve(
            Input::new()
                .with_prompt(prompt.prompt_str)
                .allow_empty(allow_empty)
                .default(s)
                .show_default(true)
                .interact(),
        ),
        None => resolve(
            Input::new()
                .with_prompt(prompt.prompt_str)
                .allow_empty(allow_empty)
                .show_default(false)
                .interact(),
        ),
    };
    match ask {
        Some(answer) => {
            if db.set(prompt.db_key, &answer).is_err() {
                eprintln!("Failed writing value {} to prickle db", prompt.db_key)
            }
            Some(answer)
        }
        None => None,
    }
}

fn resolve(result: StdResult<String, std::io::Error>) -> Option<String> {
    match result {
        Ok(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::fs::remove_file;

    use super::*;

    #[test]
    fn test_db() {
        remove_file(PROP_FILE).unwrap_or(());
        let password_string = "world".to_owned();
        let user_string = "hello".to_owned();
        let bool_val = true;
        {
            let mut db = get_db();
            db.set("foo", &password_string).unwrap();
        }
        {
            let mut db = get_db();
            db.set("bar", &user_string).unwrap();
        }
        {
            let mut db = get_db();
            db.set("poo", &bool_val).unwrap();
        }
        let db = get_db();
        let user: Option<String> = db.get("bar");
        assert_eq!(user, Some(user_string));
        let password: Option<String> = db.get("foo");
        assert_eq!(password, Some(password_string));
        let verbose: Option<bool> = db.get("poo");
        assert_eq!(verbose, Some(bool_val));
    }
}
