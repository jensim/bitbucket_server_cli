#![deny(warnings)]
#![forbid(unsafe_code)]

extern crate reqwest;
#[macro_use]
extern crate serde;

pub mod cloner;
pub mod types;

mod bitbucket;
pub mod completion;
mod git;
mod input;
mod prompts;
pub mod util;
