use super::types::Repo;
use rayon::prelude::*;

pub fn git_going(repos: Vec<Repo>) {
    repos.into_par_iter().for_each(|repo| clone_or_update(repo));
}

fn clone_or_update(r: Repo) {
    if dir_exists(&r.name) {
        update(r);
    } else {
        clone(r)
    }
}

fn dir_exists(name: &String) -> bool {
    // TODO
    return false;
}

fn update(r: Repo) {}
fn clone(r: Repo) {}
