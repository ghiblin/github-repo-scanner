# Rule ID Format — Design Spec

**Spec:** 002
**Status:** Approved
**Amends:** [spec 001](./001-github-malware-scanner-design.md) §`rules` crate — `RuleId`
**References:** [constitution](../constitution.md) · [tech-stack](../tech-stack.md) · [rules](../rules.md)

---

## Overview

Replaces the UUID v7 `RuleId` convention (spec 001) with a human-authored, human-readable format: `<LANG>-<NNN>` (e.g. `NODE-001`). Rule IDs are written by hand in TOML rule files and never generated at runtime.

The `Language` enum is the single source of truth for the prefix mapping. Both `RuleId` validation and `RuleSet` validation depend on it.

---

## Architecture

This change is confined to the `rules` crate. No other crate changes.

```
rules crate
├── Language          (prefix() / from_prefix())
├── RuleId            (parse() only — no new())
└── RuleSet::load()   (cross-checks RuleId prefix vs rule language field)
```

---

## Bounded Context — `rules` crate

### `Language` enum

`Language` owns the canonical prefix mapping. It gains two methods:

```rust
pub enum Language {
    NodeJs,
    Any,
}

impl Language {
    /// Returns the canonical uppercase prefix for this language.
    pub fn prefix(&self) -> &'static str {
        match self {
            Language::NodeJs => "NODE",
            Language::Any    => "ANY",
        }
    }

    /// Reverse lookup: returns the Language whose prefix matches `s`, or None.
    pub fn from_prefix(s: &str) -> Option<Self> {
        match s {
            "NODE" => Some(Language::NodeJs),
            "ANY"  => Some(Language::Any),
            _      => None,
        }
    }
}
```

Adding a new language requires one new variant and one new match arm in each method — no other code changes.

---

### `RuleId` Value Object

```rust
pub struct RuleId(String);
```

**Format:** `<LANG>-<NNN>` where `<LANG>` is an uppercase string and `<NNN>` is a 3-digit zero-padded integer (e.g. `NODE-001`, `ANY-003`).

**`parse()` contract** — validates two things, in order:

1. **Format**: matches `^[A-Z]+-\d{3}$`
2. **Known prefix**: the extracted prefix resolves via `Language::from_prefix()`

```rust
impl RuleId {
    pub fn parse(s: &str) -> Result<Self, RuleError> {
        // validate format
        // validate prefix is a known Language prefix
        // return Ok(RuleId(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

There is no `new()` — `RuleId` is always parsed from a TOML-authored string.

---

### `RuleSet` validation

`RuleSet::load()` performs a cross-check after all rules are parsed:

- For each rule, extract the prefix from `rule.id`
- Resolve it to a `Language` via `Language::from_prefix()`
- Assert it matches `rule.language`
- If not, return `RuleError::RuleIdLanguageMismatch`

This is a separate validation step from `RuleId::parse()`. A `RuleId` can be well-formed (`NODE-001`) yet still mismatch the rule's declared language (`Any`).

---

## Error Handling

New error variants added to `RuleError` in the `rules` crate:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    // existing variants ...

    #[error("invalid rule ID format '{id}': expected <LANG>-<NNN> with a known language prefix")]
    InvalidRuleId { id: String },

    #[error("rule ID '{id}' prefix '{actual_prefix}' does not match language '{expected_prefix}'")]
    RuleIdLanguageMismatch {
        id: String,
        actual_prefix: String,
        expected_prefix: String,
    },
}
```

Both errors are hard failures at load time — the scanner exits before any analysis begins.

---

## Testing Strategy

Follows `rules.md` TDD requirements.

### `Language` unit tests

| Case | Expected |
|---|---|
| `Language::NodeJs.prefix()` | `"NODE"` |
| `Language::Any.prefix()` | `"ANY"` |
| `Language::from_prefix("NODE")` | `Some(Language::NodeJs)` |
| `Language::from_prefix("ANY")` | `Some(Language::Any)` |
| `Language::from_prefix("PY")` | `None` |
| `Language::from_prefix("")` | `None` |

### `RuleId::parse()` unit tests

| Case | Expected |
|---|---|
| `"NODE-001"` | `Ok` |
| `"ANY-042"` | `Ok` |
| `"node-001"` (lowercase) | `Err(InvalidRuleId)` |
| `"NODE-1"` (too short) | `Err(InvalidRuleId)` |
| `"NODE001"` (no hyphen) | `Err(InvalidRuleId)` |
| `"FOO-001"` (unknown prefix) | `Err(InvalidRuleId)` |
| `""` | `Err(InvalidRuleId)` |

### `RuleSet` cross-check integration tests

| Case | Expected |
|---|---|
| Rule `id = "NODE-001"`, `language = "NodeJs"` | `Ok` |
| Rule `id = "ANY-001"`, `language = "Any"` | `Ok` |
| Rule `id = "NODE-001"`, `language = "Any"` | `Err(RuleIdLanguageMismatch)` |
| Rule `id = "ANY-001"`, `language = "NodeJs"` | `Err(RuleIdLanguageMismatch)` |

---

## Out of Scope

- Other Value Object newtypes (e.g. a future `ScanId`) — they continue to follow the UUID v7 convention in `rules.md`
- IDs longer than 3 digits (`NODE-0001`)
- Multi-language rules (a rule cannot declare more than one `language`)
- Rule ID uniqueness across multiple TOML files loaded together
