use std::result::Result as StdResult;
use std::str::FromStr;

use dialoguer::{Confirmation, Input, PasswordInput, theme::ColorfulTheme};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};

use crate::prompts::{Prompt, PROMPT_BB_PASSWORD, PROMPT_BB_PROJECT_ALL, PROMPT_BB_PROJECT_ONE, PROMPT_BB_SERVER, PROMPT_BB_USERNAME, PROMPT_RESET_STATE, PROMPT_THREAD_COUNT, PROMPT_VERBOSE};
use crate::types::Opts;

const PROP_FILE:&str = ".bitbucket_server_cli.db";

pub fn opts() -> Opts {
    let bit_bucket_server: String = get_with_default(&PROMPT_BB_SERVER, None);
    let bit_bucket_username: String = get_with_default(&PROMPT_BB_USERNAME, None);
    let bit_bucket_password: Option<String> = get_password(&PROMPT_BB_PASSWORD);
    let bit_bucket_project_all: bool = get_bool(&PROMPT_BB_PROJECT_ALL, true);
    let bit_bucket_project_key: Option<String> = match bit_bucket_project_all {
        true => None,
        false => Some(get_with_default(&PROMPT_BB_PROJECT_ONE, None))
    };
    let thread_count: usize = get_valid_type(&PROMPT_THREAD_COUNT, Some("3".to_owned()));
    let reset_state: bool = get_bool(&PROMPT_RESET_STATE, false);
    let verbose: bool = get_bool(&PROMPT_VERBOSE, false);

    Opts {
        bit_bucket_project_all,
        bit_bucket_project_key,
        bit_bucket_server,
        bit_bucket_username,
        bit_bucket_password,
        thread_count,
        reset_state,
        verbose,
    }
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

fn get_valid_type<T>(prompt: &Prompt, default: Option<String>) -> T
    where
        T: serde::Serialize,
        T: FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let until = 5;
    for _i in 1..until {
        let s: String = get_with_default(prompt, default.clone());
        let r: std::result::Result<T, <T as std::str::FromStr>::Err> = s.parse::<T>();
        if r.is_err() {
            continue;
        } else {
            return r.unwrap();
        }
    }
    eprintln!("Failed to read value after {} attempts.", until);
    std::process::exit(1);
}

fn get_with_default(prompt: &Prompt, default: Option<String>) -> String {
    let mut db = get_db();
    let read_val: Option<String> = db.get(prompt.db_key).or(default);
    let mut ask: Option<String> = None;
    while ask.is_none() || ask.clone().unwrap().trim().is_empty() {
        match read_val.clone() {
            Some(s) => ask = resolve(Input::new().with_prompt(prompt.prompt_str).allow_empty(false).default(s).show_default(true).interact()),
            None => ask = resolve(Input::new().with_prompt(prompt.prompt_str).allow_empty(true).show_default(false).interact())
        }
    }
    let answer = ask.unwrap();
    match db.set(prompt.db_key, &answer) {
        Err(_) => eprintln!("Failed writing value to prickle db"),
        _ => {},
    };
    answer
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
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn test_db() {
        remove_file(PROP_FILE).unwrap_or(());
        let password_string = "world".to_owned();
        let user_string = "hello".to_owned();
        let bool_val = true;
        {
            let mut db = get_db();
            db.set(&PROMPT_BB_PASSWORD.db_key, &password_string).unwrap();
        }
        {
            let mut db = get_db();
            db.set(&PROMPT_BB_USERNAME.db_key, &user_string).unwrap();
        }
        {
            let mut db = get_db();
            db.set(&PROMPT_VERBOSE.db_key, &bool_val).unwrap();
        }
        let db = get_db();
        let user: Option<String> = db.get(&PROMPT_BB_USERNAME.db_key);
        assert_eq!(user, Some(user_string));
        let password: Option<String> = db.get(&PROMPT_BB_PASSWORD.db_key);
        assert_eq!(password, Some(password_string));
        let verbose: Option<bool> = db.get(&PROMPT_VERBOSE.db_key);
        assert_eq!(verbose, Some(bool_val));
    }
}
