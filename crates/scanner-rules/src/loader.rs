use std::{collections::HashSet, path::Path};
use crate::{error::RulesError, models::RuleSet};

#[derive(serde::Deserialize)]
struct RuleSetFile {
    rules: Vec<crate::models::Rule>,
}

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
