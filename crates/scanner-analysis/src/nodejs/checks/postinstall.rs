use crate::models::Finding;
use scanner_repository::RepoFile;
use scanner_rules::Rule;

#[must_use]
pub fn check(file: &RepoFile, rule: &Rule, keys: &[String]) -> Vec<Finding> {
    if file
        .path
        .file_name()
        .is_none_or(|n| n != "package.json")
    {
        return vec![];
    }
    let scanner_repository::FileContent::Text(ref content) = file.content else {
        return vec![];
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(content) else {
        return vec![];
    };
    let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) else {
        return vec![];
    };
    keys.iter()
        .filter(|k| scripts.contains_key(*k))
        .map(|k| Finding {
            rule_id: rule.id.clone(),
            severity: rule.severity.clone(),
            file: file.path.clone(),
            line: None,
            message: format!("lifecycle script '{k}' found in package.json"),
            snippet: scripts[k].as_str().map(std::borrow::ToOwned::to_owned),
        })
        .collect()
}
