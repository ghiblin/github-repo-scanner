use std::sync::Arc;

use scanner_repository::RepoSnapshot;
use scanner_rules::RuleSet;

use crate::models::Finding;

pub trait Analyzer {
    fn analyze(&self, snapshot: &RepoSnapshot, rules: &Arc<RuleSet>) -> Vec<Finding>;
}
