pub struct Prompt {
    pub db_key: &'static str,
    pub prompt_str: &'static str,
}

pub const PROMPT_BB_PROJECT_ALL: Prompt = Prompt {
    db_key: "bb_project_all",
    prompt_str: "Clone/update all found projects with repos",
};
pub const PROMPT_BB_PROJECT_ONE: Prompt = Prompt {
    db_key: "bb_project_one",
    prompt_str: "Clone/update single project key",
};
pub const PROMPT_BB_SERVER: Prompt = Prompt {
    db_key: "bb_server",
    prompt_str: "BitBucket server address",
};
pub const PROMPT_BB_USERNAME: Prompt = Prompt {
    db_key: "bb_username",
    prompt_str: "BitBucket username",
};
pub const PROMPT_BB_PASSWORD: Prompt = Prompt {
    db_key: "bb_password",
    prompt_str: "BitBucket password",
};
pub const PROMPT_THREAD_COUNT: Prompt = Prompt {
    db_key: "thread_count",
    prompt_str: "Thread count",
};
pub const PROMPT_RESET_STATE: Prompt = Prompt {
    db_key: "reset_state",
    prompt_str: "Reset state",
};
pub const PROMPT_VERBOSE: Prompt = Prompt {
    db_key: "verbose",
    prompt_str: "Verbose output",
};
