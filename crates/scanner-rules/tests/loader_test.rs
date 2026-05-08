use scanner_rules::loader::load;
use std::path::Path;

#[test]
fn loads_valid_ruleset() {
    let path = Path::new("tests/fixtures/valid.toml");
    let ruleset = load(path).expect("should load");
    assert_eq!(ruleset.rules.len(), 1);
    assert_eq!(ruleset.rules[0].name, "Test rule");
}

#[test]
fn rejects_invalid_pattern_type() {
    let path = Path::new("tests/fixtures/invalid_pattern_type.toml");
    assert!(load(path).is_err());
}

#[test]
fn rejects_duplicate_ids() {
    let path = Path::new("tests/fixtures/duplicate_ids.toml");
    let err = load(path).unwrap_err();
    assert!(err.to_string().contains("duplicate"));
}

#[test]
fn rejects_missing_required_field() {
    let path = Path::new("tests/fixtures/missing_field.toml");
    assert!(load(path).is_err());
}

#[test]
fn loads_default_nodejs_ruleset() {
    let path = std::path::Path::new("../../rules/nodejs.toml");
    let ruleset = load(path).expect("default ruleset must be valid");
    assert_eq!(ruleset.rules.len(), 11);
}

#[test]
fn loads_default_vscode_ruleset() {
    let path = std::path::Path::new("../../rules/vscode.toml");
    let ruleset = load(path).expect("vscode ruleset must be valid");
    assert_eq!(ruleset.rules.len(), 4);
}

#[test]
fn rejects_node_id_on_any_language() {
    let path = Path::new("tests/fixtures/id_language_mismatch.toml");
    let err = load(path).unwrap_err();
    assert!(
        err.to_string().contains("does not match"),
        "unexpected error: {err}"
    );
}

#[test]
fn rejects_any_id_on_nodejs_language() {
    let path = Path::new("tests/fixtures/any_id_language_mismatch.toml");
    let err = load(path).unwrap_err();
    assert!(
        err.to_string().contains("does not match"),
        "unexpected error: {err}"
    );
}

#[test]
fn loads_valid_vscode_ruleset() {
    let path = Path::new("tests/fixtures/valid_vscode.toml");
    let ruleset = load(path).expect("should load VSCODE ruleset");
    assert_eq!(ruleset.rules.len(), 1);
    assert_eq!(ruleset.rules[0].name, "Test VsCode rule");
}
