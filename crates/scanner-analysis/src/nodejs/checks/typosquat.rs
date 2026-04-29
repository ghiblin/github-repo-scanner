use crate::models::Finding;
use scanner_repository::{FileContent, RepoFile};
use scanner_rules::Rule;
use strsim::osa_distance;

#[must_use]
pub fn check(file: &RepoFile, rule: &Rule, known_packages: &[String]) -> Vec<Finding> {
    if file.path.file_name().is_some_and(|n| n != "package.json") {
        return vec![];
    }
    let FileContent::Text(ref content) = file.content else {
        return vec![];
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(content) else {
        return vec![];
    };

    let dep_sections = ["dependencies", "devDependencies", "peerDependencies"];
    let mut findings = Vec::new();

    for section in dep_sections {
        let Some(deps) = json.get(section).and_then(|d| d.as_object()) else {
            continue;
        };
        for dep_name in deps.keys() {
            for known in known_packages {
                if dep_name == known {
                    continue;
                }
                if osa_distance(dep_name, known) == 1 {
                    findings.push(Finding {
                        rule_id: rule.id.clone(),
                        severity: rule.severity.clone(),
                        file: file.path.clone(),
                        line: None,
                        message: format!(
                            "'{dep_name}' is 1 character away from known package '{known}'"
                        ),
                        snippet: Some(dep_name.clone()),
                    });
                }
            }
        }
    }
    findings
}
