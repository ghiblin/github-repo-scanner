use crate::models::Finding;
use scanner_repository::RepoFile;
use scanner_rules::Rule;

#[must_use]
pub fn check(_file: &RepoFile, _rule: &Rule, _pattern: &str) -> Vec<Finding> {
    vec![]
}

#[must_use]
pub fn check_file_match(_file: &RepoFile, _rule: &Rule, _glob: &str) -> Vec<Finding> {
    vec![]
}
