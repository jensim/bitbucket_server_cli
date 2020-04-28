#![forbid(unsafe_code)]
extern crate reqwest;
#[macro_use]
extern crate serde;

pub mod types;
pub mod cloner;

mod prompts;
mod bitbucket;
mod git;
mod input;
