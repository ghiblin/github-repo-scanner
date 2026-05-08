use crate::{analyzer::Analyzer, models::Finding};
use scanner_repository::RepoSnapshot;
use scanner_rules::{Pattern, RuleSet};
use std::sync::Arc;

pub mod checks;

pub struct VsCodeAnalyzer;

impl Analyzer for VsCodeAnalyzer {
    fn analyze(&self, snapshot: &RepoSnapshot, rules: &Arc<RuleSet>) -> Vec<Finding> {
        let mut findings = Vec::new();
        for rule in &rules.rules {
            for file in &snapshot.files {
                let new = match &rule.pattern {
                    Pattern::Regex { value } => checks::check(file, rule, value),
                    Pattern::ExtensionIdCheck => checks::extension_id_check(file, rule),
                    _ => vec![],
                };
                findings.extend(new);
            }
        }
        findings
    }
}
