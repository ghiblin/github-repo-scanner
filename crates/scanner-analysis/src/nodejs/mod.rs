use crate::{analyzer::Analyzer, models::Finding};
use scanner_repository::RepoSnapshot;
use scanner_rules::{Pattern, RuleSet};
use std::sync::Arc;

pub mod checks;

pub struct NodeJsAnalyzer;

impl Analyzer for NodeJsAnalyzer {
    fn analyze(&self, snapshot: &RepoSnapshot, rules: &Arc<RuleSet>) -> Vec<Finding> {
        let mut findings = Vec::new();
        for rule in &rules.rules {
            for file in &snapshot.files {
                let new = match &rule.pattern {
                    Pattern::ScriptKey { keys } => checks::postinstall::check(file, rule, keys),
                    Pattern::Regex { value } => checks::obfuscation::check(file, rule, value),
                    Pattern::TypoSquat { known_packages } => {
                        checks::typosquat::check(file, rule, known_packages)
                    }
                    Pattern::FileMatch { glob } => {
                        checks::shell::check_file_match(file, rule, glob)
                    }
                    Pattern::ExtensionIdCheck | Pattern::TerminalEnvInjectionCheck => vec![],
                };
                findings.extend(new);
            }
        }
        findings
    }
}
