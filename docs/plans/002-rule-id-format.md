# Rule ID Format — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace UUID v7 `RuleId` with the human-readable `<LANG>-<NNN>` format defined in spec 002. `Language` becomes the single source of truth for the prefix mapping. `RuleId::parse()` validates both format and known prefix. The loader adds a cross-check that the ID prefix matches the rule's declared language.
**Architecture:** All logic changes are confined to `scanner-rules`. TOML rule files and `scanner-analysis` test fixtures are updated in follow-up tasks.
**Tech Stack:** Rust stable, `scanner-rules` crate, `thiserror`. The `uuid` crate is removed from `scanner-rules/Cargo.toml`.

---

## File Map

```
crates/scanner-rules/src/models.rs                                add Language::prefix() / from_prefix()
crates/scanner-rules/src/rule_id.rs                               replace UUID v7 logic with <LANG>-<NNN> parser
crates/scanner-rules/src/error.rs                                 add RulesError::InvalidRuleId and LanguageMismatch
crates/scanner-rules/src/loader.rs                                add ID format validation and language cross-check
crates/scanner-rules/Cargo.toml                                   remove uuid dependency
crates/scanner-rules/tests/fixtures/valid.toml                    update ID
crates/scanner-rules/tests/fixtures/duplicate_ids.toml            update IDs
crates/scanner-rules/tests/fixtures/invalid_pattern_type.toml     update ID
crates/scanner-rules/tests/fixtures/missing_field.toml            update ID
crates/scanner-rules/tests/fixtures/id_language_mismatch.toml     new
crates/scanner-rules/tests/fixtures/any_id_language_mismatch.toml new
crates/scanner-rules/tests/loader_test.rs                         add two new test cases
rules/nodejs.toml                                                  update IDs to NODE-001..NODE-006
crates/scanner-analysis/tests/node_001_test.rs                    update RuleId::parse call
crates/scanner-analysis/tests/node_002_test.rs                    update RuleId::parse call
crates/scanner-analysis/tests/node_003_test.rs                    update RuleId::parse call
crates/scanner-analysis/tests/node_004_test.rs                    update RuleId::parse call
crates/scanner-analysis/tests/node_005_test.rs                    update RuleId::parse call
crates/scanner-analysis/tests/node_006_test.rs                    update RuleId::parse call
```

---

## Task 1: Language prefix methods

**Files:**
- Modify: `crates/scanner-rules/src/models.rs`

- [ ] **Step 1: Write failing tests**

Add to the `#[cfg(test)] mod tests` block in `models.rs`, and replace the existing `ruleset_holds_rules` test (it uses a UUID v7 `RuleId` that will no longer be valid):

```rust
#[test]
fn nodejs_prefix_is_node() {
    assert_eq!(Language::NodeJs.prefix(), "NODE");
}

#[test]
fn any_prefix_is_any() {
    assert_eq!(Language::Any.prefix(), "ANY");
}

#[test]
fn from_prefix_node_returns_nodejs() {
    assert_eq!(Language::from_prefix("NODE"), Some(Language::NodeJs));
}

#[test]
fn from_prefix_any_returns_any() {
    assert_eq!(Language::from_prefix("ANY"), Some(Language::Any));
}

#[test]
fn from_prefix_unknown_returns_none() {
    assert_eq!(Language::from_prefix("FOO"), None);
}

#[test]
fn from_prefix_empty_returns_none() {
    assert_eq!(Language::from_prefix(""), None);
}

// Replace the old ruleset_holds_rules test:
#[test]
fn ruleset_holds_rules() {
    let id = crate::RuleId::parse("NODE-001").unwrap();
    let rule = Rule {
        id,
        name: "Test rule".to_owned(),
        description: "desc".to_owned(),
        severity: Severity::High,
        language: Language::NodeJs,
        pattern: Pattern::Regex {
            value: "foo".to_owned(),
        },
    };
    let ruleset = RuleSet { rules: vec![rule] };
    assert_eq!(ruleset.rules.len(), 1);
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo test -p scanner-rules
```

Expected: compile errors — `prefix` and `from_prefix` methods do not exist on `Language`.

- [ ] **Step 3: Implement**

Add an `impl Language` block in `models.rs` after the enum definition:

```rust
impl Language {
    #[must_use]
    pub fn prefix(&self) -> &'static str {
        match self {
            Language::NodeJs => "NODE",
            Language::Any    => "ANY",
        }
    }

    #[must_use]
    pub fn from_prefix(s: &str) -> Option<Self> {
        match s {
            "NODE" => Some(Language::NodeJs),
            "ANY"  => Some(Language::Any),
            _      => None,
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p scanner-rules
```

Expected: the 6 new `Language` tests pass. `ruleset_holds_rules` still fails at runtime (UUID v7 parser rejects `"NODE-001"`) — that is expected and fixed in Task 2.

- [ ] **Step 5: Commit**

```bash
git add crates/scanner-rules/src/models.rs
git commit -m "feat(scanner-rules): add Language::prefix() and Language::from_prefix()"
```

---

## Task 2: Replace RuleId

**Files:**
- Modify: `crates/scanner-rules/src/rule_id.rs`
- Modify: `crates/scanner-rules/Cargo.toml`

- [ ] **Step 1: Write failing tests**

Replace the entire `#[cfg(test)] mod tests` block in `rule_id.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_valid_node_id() {
        assert!(RuleId::parse("NODE-001").is_ok());
    }

    #[test]
    fn parse_accepts_valid_any_id() {
        assert!(RuleId::parse("ANY-042").is_ok());
    }

    #[test]
    fn parse_rejects_lowercase_prefix() {
        assert!(RuleId::parse("node-001").is_err());
    }

    #[test]
    fn parse_rejects_short_number() {
        assert!(RuleId::parse("NODE-1").is_err());
    }

    #[test]
    fn parse_rejects_missing_hyphen() {
        assert!(RuleId::parse("NODE001").is_err());
    }

    #[test]
    fn parse_rejects_unknown_prefix() {
        assert!(RuleId::parse("FOO-001").is_err());
    }

    #[test]
    fn parse_rejects_empty_string() {
        assert!(RuleId::parse("").is_err());
    }

    #[test]
    fn display_shows_full_string() {
        let id = RuleId::parse("NODE-001").unwrap();
        assert_eq!(id.to_string(), "NODE-001");
    }

    #[test]
    fn as_str_returns_inner_string() {
        let id = RuleId::parse("NODE-001").unwrap();
        assert_eq!(id.as_str(), "NODE-001");
    }
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo test -p scanner-rules rule_id
```

Expected: `parse_accepts_valid_node_id` and `parse_accepts_valid_any_id` fail — the UUID v7 parser rejects the new format.

- [ ] **Step 3: Implement**

Replace the full contents of `crates/scanner-rules/src/rule_id.rs`:

```rust
use crate::models::Language;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RuleId(String);

#[derive(Debug, thiserror::Error)]
pub enum RuleIdError {
    #[error("invalid rule ID format '{id}': expected <LANG>-<NNN> with a known language prefix")]
    InvalidRuleId { id: String },
}

impl RuleId {
    /// # Errors
    /// Returns `InvalidRuleId` if `s` does not match `<LANG>-<NNN>` with a known language prefix.
    pub fn parse(s: &str) -> Result<Self, RuleIdError> {
        let err = || RuleIdError::InvalidRuleId { id: s.to_owned() };
        let (prefix, number) = s.split_once('-').ok_or_else(err)?;
        if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(err());
        }
        if number.len() != 3 || !number.chars().all(|c| c.is_ascii_digit()) {
            return Err(err());
        }
        if Language::from_prefix(prefix).is_none() {
            return Err(err());
        }
        Ok(Self(s.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

Remove the `uuid` dependency from `crates/scanner-rules/Cargo.toml`:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
toml = "0.8"
thiserror = "2"
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p scanner-rules
```

Expected: all `rule_id` tests pass and `ruleset_holds_rules` passes. Some loader tests may fail because TOML fixtures still contain UUID v7 IDs — that is expected and addressed in Task 4.

- [ ] **Step 5: Commit**

```bash
git add crates/scanner-rules/src/rule_id.rs crates/scanner-rules/Cargo.toml
git commit -m "feat(scanner-rules): replace RuleId UUID v7 with <LANG>-<NNN> format"
```

---

## Task 3: Loader cross-check

**Files:**
- Modify: `crates/scanner-rules/src/error.rs`
- Modify: `crates/scanner-rules/src/loader.rs`
- Modify: `crates/scanner-rules/tests/loader_test.rs`
- Create: `crates/scanner-rules/tests/fixtures/id_language_mismatch.toml`
- Create: `crates/scanner-rules/tests/fixtures/any_id_language_mismatch.toml`

- [ ] **Step 1: Write failing tests**

Add to `crates/scanner-rules/tests/loader_test.rs`:

```rust
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
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo test -p scanner-rules loader
```

Expected: both new tests fail — the fixture files do not exist yet, so `load()` returns an I/O error rather than a mismatch error.

- [ ] **Step 3: Implement**

Create `crates/scanner-rules/tests/fixtures/id_language_mismatch.toml`:

```toml
[[rules]]
id = "NODE-001"
name = "Mismatch rule"
description = "ID prefix is NODE but language is Any."
severity = "High"
language = "Any"
pattern = { type = "Regex", value = "foo" }
```

Create `crates/scanner-rules/tests/fixtures/any_id_language_mismatch.toml`:

```toml
[[rules]]
id = "ANY-001"
name = "Mismatch rule"
description = "ID prefix is ANY but language is NodeJs."
severity = "High"
language = "NodeJs"
pattern = { type = "Regex", value = "foo" }
```

Replace `crates/scanner-rules/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RulesError {
    #[error("failed to read ruleset file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse ruleset TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("duplicate rule ID: {0}")]
    DuplicateId(String),
    #[error(transparent)]
    InvalidRuleId(#[from] crate::rule_id::RuleIdError),
    #[error("rule ID '{id}' prefix '{actual_prefix}' does not match language '{expected_prefix}'")]
    LanguageMismatch {
        id: String,
        actual_prefix: String,
        expected_prefix: String,
    },
}
```

Replace `crates/scanner-rules/src/loader.rs`:

```rust
use crate::{error::RulesError, models::RuleSet, rule_id::RuleId};
use std::{collections::HashSet, path::Path};

#[derive(serde::Deserialize)]
struct RuleSetFile {
    rules: Vec<crate::models::Rule>,
}

/// # Errors
/// Returns an error if the file cannot be read, the TOML is malformed, a rule ID has an invalid
/// format, a rule ID prefix does not match its language, or duplicate rule IDs are found.
pub fn load(path: &Path) -> Result<RuleSet, RulesError> {
    let contents = std::fs::read_to_string(path)?;
    let file: RuleSetFile = toml::from_str(&contents)?;
    let mut seen_ids = HashSet::new();
    for rule in &file.rules {
        RuleId::parse(rule.id.as_str())?;
        if !seen_ids.insert(rule.id.as_str().to_owned()) {
            return Err(RulesError::DuplicateId(rule.id.to_string()));
        }
        let actual_prefix = rule.id.as_str().split('-').next().unwrap_or("");
        let expected_prefix = rule.language.prefix();
        if actual_prefix != expected_prefix {
            return Err(RulesError::LanguageMismatch {
                id: rule.id.to_string(),
                actual_prefix: actual_prefix.to_owned(),
                expected_prefix: expected_prefix.to_owned(),
            });
        }
    }
    Ok(RuleSet { rules: file.rules })
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p scanner-rules loader
```

Expected: all loader tests pass including the 2 new mismatch tests. `loads_valid_ruleset` and `loads_default_nodejs_ruleset` will fail until Task 4 updates the TOML files — that is expected.

- [ ] **Step 5: Commit**

```bash
git add crates/scanner-rules/src/error.rs \
        crates/scanner-rules/src/loader.rs \
        crates/scanner-rules/tests/loader_test.rs \
        crates/scanner-rules/tests/fixtures/id_language_mismatch.toml \
        crates/scanner-rules/tests/fixtures/any_id_language_mismatch.toml
git commit -m "feat(scanner-rules): add language/prefix cross-check in RuleSet loader"
```

---

## Task 4: Update TOML files

**Files:**
- Modify: `crates/scanner-rules/tests/fixtures/valid.toml`
- Modify: `crates/scanner-rules/tests/fixtures/duplicate_ids.toml`
- Modify: `crates/scanner-rules/tests/fixtures/invalid_pattern_type.toml`
- Modify: `crates/scanner-rules/tests/fixtures/missing_field.toml`
- Modify: `rules/nodejs.toml`

- [ ] **Step 1: Update test fixtures**

`crates/scanner-rules/tests/fixtures/valid.toml`:

```toml
[[rules]]
id = "NODE-001"
name = "Test rule"
description = "A valid test rule."
severity = "High"
language = "NodeJs"
pattern = { type = "Regex", value = "eval" }
```

`crates/scanner-rules/tests/fixtures/duplicate_ids.toml`:

```toml
[[rules]]
id = "NODE-001"
name = "Rule A"
description = "First rule."
severity = "Low"
language = "NodeJs"
pattern = { type = "Regex", value = "foo" }

[[rules]]
id = "NODE-001"
name = "Rule B"
description = "Duplicate ID."
severity = "Low"
language = "NodeJs"
pattern = { type = "Regex", value = "bar" }
```

`crates/scanner-rules/tests/fixtures/invalid_pattern_type.toml`:

```toml
[[rules]]
id = "NODE-001"
name = "Bad pattern"
description = "Unknown pattern type."
severity = "High"
language = "NodeJs"
pattern = { type = "Unknown", value = "foo" }
```

`crates/scanner-rules/tests/fixtures/missing_field.toml`:

```toml
[[rules]]
id = "NODE-001"
name = "Missing severity"
language = "NodeJs"
pattern = { type = "Regex", value = "foo" }
```

- [ ] **Step 2: Update `rules/nodejs.toml`**

Replace the 6 UUID v7 IDs with `NODE-001` through `NODE-006`. Keep every other field unchanged:

```toml
[[rules]]
id = "NODE-001"
name = "Malicious postinstall script"
description = "npm lifecycle scripts (postinstall, preinstall, install) can execute arbitrary code when the package is installed. This is a common malware vector."
severity = "Critical"
language = "NodeJs"
pattern = { type = "ScriptKey", keys = ["postinstall", "preinstall", "install"] }

[[rules]]
id = "NODE-002"
name = "Obfuscated JavaScript"
description = "Detects common obfuscation patterns: eval(atob(...)), hex-encoded strings, and packed/mangled code that hides intent."
severity = "High"
language = "NodeJs"
pattern = { type = "Regex", value = 'eval\s*\(\s*atob|\\x[0-9a-fA-F]{2}|_0x[0-9a-fA-F]+' }

[[rules]]
id = "NODE-003"
name = "Suspicious outbound network call"
description = "Detects hardcoded external IP addresses in source code, which may indicate data exfiltration."
severity = "High"
language = "NodeJs"
pattern = { type = "Regex", value = 'https?://(?:10|172|192)\.[0-9]{1,3}' }

[[rules]]
id = "NODE-004"
name = "Shell execution in source"
description = "Calls to exec(), execSync(), spawn(), or child_process are commonly used to run arbitrary system commands."
severity = "High"
language = "NodeJs"
pattern = { type = "Regex", value = "require\\(['\"]child_process['\"]\\)|execSync|spawnSync" }

[[rules]]
id = "NODE-005"
name = "Hardcoded secret or token"
description = "Detects patterns matching API keys, tokens, or private keys committed directly in source."
severity = "Critical"
language = "NodeJs"
pattern = { type = "Regex", value = "(?i)(api_key|secret|token|private_key)\\s*=\\s*['\"][A-Za-z0-9_\\-]{16,}" }

[[rules]]
id = "NODE-006"
name = "Typosquatting risk in package.json"
description = "Flags dependency names with a Levenshtein distance of 1 from a known popular package (e.g. 'expres', 'lodahs')."
severity = "Medium"
language = "NodeJs"
pattern = { type = "TypoSquat", known_packages = ["express", "lodash", "react", "axios", "moment", "webpack", "babel", "typescript", "eslint", "prettier"] }
```

- [ ] **Step 3: Run tests to verify they pass**

```bash
cargo test -p scanner-rules
```

Expected: all 13 `scanner-rules` tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/scanner-rules/tests/fixtures/ rules/nodejs.toml
git commit -m "chore(rules): migrate rule IDs from UUID v7 to NODE-NNN format"
```

---

## Task 5: Fix analysis tests

**Files:**
- Modify: `crates/scanner-analysis/tests/node_001_test.rs`
- Modify: `crates/scanner-analysis/tests/node_002_test.rs`
- Modify: `crates/scanner-analysis/tests/node_003_test.rs`
- Modify: `crates/scanner-analysis/tests/node_004_test.rs`
- Modify: `crates/scanner-analysis/tests/node_005_test.rs`
- Modify: `crates/scanner-analysis/tests/node_006_test.rs`

- [ ] **Step 1: Run to verify failure**

```bash
cargo test -p scanner-analysis
```

Expected: all tests fail — `RuleId::parse("rul_018f1234-...")` is rejected by the new parser.

- [ ] **Step 2: Apply changes**

In each file, on line 21, replace the UUID v7 string with the human-readable ID:

| File | Replace with |
|---|---|
| `node_001_test.rs` | `RuleId::parse("NODE-001").unwrap()` |
| `node_002_test.rs` | `RuleId::parse("NODE-002").unwrap()` |
| `node_003_test.rs` | `RuleId::parse("NODE-003").unwrap()` |
| `node_004_test.rs` | `RuleId::parse("NODE-004").unwrap()` |
| `node_005_test.rs` | `RuleId::parse("NODE-005").unwrap()` |
| `node_006_test.rs` | `RuleId::parse("NODE-006").unwrap()` |

- [ ] **Step 3: Run tests to verify they pass**

```bash
cargo test -p scanner-analysis
```

Expected: all analysis tests pass.

- [ ] **Step 4: Run full test suite**

```bash
cargo test
```

Expected: all tests across all crates pass with zero failures.

- [ ] **Step 5: Commit**

```bash
git add crates/scanner-analysis/tests/
git commit -m "fix(scanner-analysis): update test RuleIds to NODE-NNN format"
```
