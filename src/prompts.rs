pub struct Prompt {
    pub db_key: &'static str,
    pub prompt_str: &'static str,
}

pub const PROMPT_BB_PROJECT_ALL: Prompt = Prompt {
    db_key: "bb_project_all",
    prompt_str: "Clone/update all projects",
};
pub const PROMPT_BB_PROJECT_SOME: Prompt = Prompt {
    db_key: "bb_project_some",
    prompt_str: "Clone/update some projects",
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
