use std::result::Result as StdResult;

use dialoguer::{Checkboxes, Confirmation, Input, PasswordInput, theme::ColorfulTheme};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};

use crate::prompts::{
    Prompt,
    PROMPT_BB_PASSWORD,
    PROMPT_BB_PROJECT_ALL,
    PROMPT_BB_PROJECT_SOME,
    PROMPT_BB_SERVER,
    PROMPT_BB_USERNAME};
use crate::types::{BitBucketOpts, GitOpts, Opts, Repo};

const PROP_FILE: &str = ".bitbucket_server_cli.db";

pub fn opts(opts: &Opts) -> Opts {
    let bit_bucket_server: Option<String> = match opts.bitbucket_opts.server.clone() {
        None => get_with_default(&PROMPT_BB_SERVER, None, false),
        Some(s) => Some(s)
    };
    let bit_bucket_username: Option<String> = match opts.bitbucket_opts.username.clone() {
        None => get_with_default(&PROMPT_BB_USERNAME, None, true),
        Some(s) => Some(s)
    };
    let bit_bucket_password: Option<String> = match bit_bucket_username {
        None => None,
        Some(_) => match opts.bitbucket_opts.password.clone() {
            None => get_password(&PROMPT_BB_PASSWORD),
            Some(s) => Some(s)
        }
    };
    let bit_bucket_project_all: bool = opts.git_opts.clone_all || get_bool(&PROMPT_BB_PROJECT_ALL, false);

    Opts {
        bitbucket_opts: BitBucketOpts {
            password: bit_bucket_password,
            server: bit_bucket_server,
            username: bit_bucket_username,
            concurrency: opts.bitbucket_opts.concurrency,
        },
        git_opts: GitOpts {
            reset_state: opts.git_opts.reset_state,
            clone_all: bit_bucket_project_all,
            project_keys: opts.git_opts.project_keys.clone(),
            concurrency: opts.git_opts.concurrency,
        },
        interactive: opts.interactive,
    }
}

pub fn select_projects(repos: &Vec<Repo>) -> Vec<String> {
    let mut project_keys: Vec<String> = Vec::new();
    for r in repos {
        if !project_keys.contains(&r.project_key) {
            project_keys.push(r.project_key.clone());
        }
    }
    let mut db = get_db();
    let previous: Vec<String> = db.get(PROMPT_BB_PROJECT_SOME.db_key).unwrap_or(Vec::new());
    let pre_selected: Vec<bool> = project_keys.iter().map(|key| previous.contains(key)).collect();
    let mut answer: Vec<usize> = Vec::new();
    while answer.is_empty() {
        answer = Checkboxes::new().items(&project_keys)
            .with_prompt(&PROMPT_BB_PROJECT_SOME.prompt_str)
            .defaults(&pre_selected[..])
            .interact().unwrap_or_else(|_e| {
            eprintln!("Failed handling project key selection");
            std::process::exit(1);
        });
    }
    let mut filtered: Vec<String> = Vec::new();
    for i in answer {
        filtered.push(project_keys[i].clone());
    }
    match db.set(PROMPT_BB_PROJECT_SOME.db_key, &filtered) {
        Err(_) => eprintln!("Failed writing value to prickle db"),
        _ => {}
    };
    filtered
}

fn get_db() -> PickleDb {
    PickleDb::load(PROP_FILE, PickleDbDumpPolicy::AutoDump, SerializationMethod::Yaml)
        .unwrap_or(PickleDb::new(PROP_FILE, PickleDbDumpPolicy::AutoDump, SerializationMethod::Yaml))
}

fn get_password(prompt: &Prompt) -> Option<String> {
    let password = PasswordInput::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.prompt_str)
        .allow_empty_password(true)
        .interact();
    resolve(password)
}

fn get_bool(prompt: &Prompt, default: bool) -> bool {
    let mut db = get_db();
    let read_val: bool = db.get(prompt.db_key).unwrap_or(default);
    let prompt_val = Confirmation::new()
        .with_text(prompt.prompt_str)
        .default(read_val)
        .show_default(true)
        .interact().unwrap_or(default);
    match db.set(prompt.db_key, &prompt_val) {
        Err(_) => eprintln!("Failed writing value to prickle db"),
        _ => {},
    };
    prompt_val
}

fn get_with_default(prompt: &Prompt, default: Option<String>, allow_empty: bool) -> Option<String> {
    let mut db = get_db();
    let read_val: Option<String> = db.get(prompt.db_key).or(default);
    let ask: Option<String> = match read_val.clone() {
        Some(s) => resolve(Input::new().with_prompt(prompt.prompt_str)
            .allow_empty(allow_empty).default(s).show_default(true).interact()),
        None => resolve(Input::new().with_prompt(prompt.prompt_str)
            .allow_empty(allow_empty).show_default(false).interact())
    };
    match ask {
        Some(answer) => {
            match db.set(prompt.db_key, &answer) {
                Err(_) => eprintln!("Failed writing value to prickle db"),
                _ => {},
            };
            Some(answer)
        },
        None => None,
    }
}

fn resolve(result: StdResult<String, std::io::Error>) -> Option<String> {
    match result {
        Ok(s) => if s.is_empty() {
            None
        } else {
            Some(s)
        },
        _ => {
            None
        }
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
