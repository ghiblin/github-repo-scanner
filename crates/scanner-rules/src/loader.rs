use crate::{error::RulesError, models::RuleSet};
use std::{collections::HashSet, path::Path};

#[derive(serde::Deserialize)]
struct RuleSetFile {
    rules: Vec<crate::models::Rule>,
}

/// # Errors
/// Returns an error if the file cannot be read, the TOML is malformed, or duplicate rule IDs are found.
pub fn load(path: &Path) -> Result<RuleSet, RulesError> {
    let contents = std::fs::read_to_string(path)?;
    let file: RuleSetFile = toml::from_str(&contents)?;
    let mut seen_ids = HashSet::new();
    for rule in &file.rules {
        if !seen_ids.insert(rule.id.as_str().to_owned()) {
            return Err(RulesError::DuplicateId(rule.id.to_string()));
        }
    }
    Ok(RuleSet { rules: file.rules })
}
