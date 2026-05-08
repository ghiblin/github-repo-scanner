# GitHub Token Authentication — Design Spec

**Spec:** 004
**Status:** Draft
**References:** [constitution](../constitution.md) · [tech-stack](../tech-stack.md) · [rules](../rules.md)

---

## Overview

Adds mandatory GitHub personal access token (PAT) support to the scanner. A token is required for every invocation — without one the CLI exits at startup with a clear error message. This raises the API rate limit from 60 to 5 000 requests/hour and enables scanning of private repositories when the token carries the `repo` scope.

---

## Architecture

No new crates. Changes are confined to `scanner-repository` (client construction, error variants) and `cli` (token resolution, new flag).

```
GITHUB_TOKEN env var  ──┐
                         ├─→ token resolution (cli) ─→ GithubApiClient / LocalCloneClient
--token CLI flag    ─────┘
```

---

## Bounded Contexts

### `scanner-repository` crate

#### `GithubApiClient`

Constructor signature changes to require a token:

```rust
impl GithubApiClient {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token.into()))
                .expect("token must be a valid header value"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("failed to build HTTP client");
        Self { base_url: base_url.into(), client }
    }
}
```

The `Authorization: Bearer <token>` header is set once at construction time via `reqwest::ClientBuilder::default_headers`. All subsequent requests inherit it automatically.

#### `LocalCloneClient`

The clone URL embeds the token so that `git clone` authenticates over HTTPS without requiring a credential helper:

```
https://x-access-token:{token}@github.com/{owner}/{repo}.git
```

The `LocalCloneClient` gains a `token: String` field and a matching constructor:

```rust
pub fn new(token: impl Into<String>) -> Self
```

#### `RepositoryError`

Two new variants:

```rust
#[error("authentication failed: token is invalid or expired")]
Unauthorized,

#[error("access denied to {owner}/{name}: token may lack 'repo' scope")]
Forbidden { owner: String, name: String },
```

HTTP 403 disambiguation: GitHub sets `X-RateLimit-Remaining: 0` on rate-limit responses. If the header is present and equals `0`, map to the existing `RateLimited`; otherwise map to `Forbidden`.

Full mapping:

| HTTP status | Condition | `RepositoryError` variant |
|---|---|---|
| 401 | any | `Unauthorized` |
| 403 | `X-RateLimit-Remaining: 0` | `RateLimited` |
| 403 | header absent or non-zero | `Forbidden { owner, name }` |
| 404 | any | `NotFound { owner, name }` |

---

### `cli` crate

#### Token resolution

Token resolution happens once at startup before any network call, in this precedence order:

1. `--token <TOKEN>` CLI flag (highest priority)
2. `GITHUB_TOKEN` environment variable
3. Neither present → exit immediately with:

```
error: GitHub token is required. Set GITHUB_TOKEN or pass --token <TOKEN>.
```

The resolved token is passed directly to `GithubApiClient::new` and `LocalCloneClient::new`. It is never logged or included in any output.

#### Updated CLI interface

```
repo-scanner <GITHUB_URL> [OPTIONS]

Arguments:
  <GITHUB_URL>    GitHub repository URL (https://github.com/owner/repo)

Options:
  --token <TOKEN>  GitHub personal access token (overrides GITHUB_TOKEN env var)
  --clone          Clone repo locally for deeper analysis (default: API only)
  --rules <FILE>   Path to custom TOML ruleset (default: built-in)
  --help           Print help
  --version        Print version
```

---

## Error Handling

| Scenario | Error variant | User-facing message |
|---|---|---|
| No token provided | startup error (pre-`anyhow`) | `GitHub token is required. Set GITHUB_TOKEN or pass --token <TOKEN>.` |
| Token invalid / expired | `Unauthorized` | `authentication failed: token is invalid or expired` |
| Token lacks `repo` scope | `Forbidden { owner, name }` | `access denied to {owner}/{name}: token may lack 'repo' scope` |
| Rate limit exceeded | `RateLimited` | `GitHub API rate limit exceeded` |
| Repo not found | `NotFound { owner, name }` | `repo not found: {owner}/{name}` |

---

## Testing Strategy

Follows `rules.md` TDD requirements. Tests are written before production code.

### `scanner-repository` (unit, `mockito`)

New mock cases for `GithubApiClient`:

| Case | Mock response | Expected result |
|---|---|---|
| Valid token, public repo | 200 | `Ok(RepoSnapshot)` |
| Invalid token | 401 | `Err(Unauthorized)` |
| Rate limited | 403 + `X-RateLimit-Remaining: 0` | `Err(RateLimited)` |
| Forbidden (private, no scope) | 403 (no rate-limit header) | `Err(Forbidden { … })` |

New mock cases for `LocalCloneClient`:

| Case | Expected result |
|---|---|
| Clone URL contains embedded token | URL matches `https://x-access-token:{token}@github.com/…` |

### `cli` (integration, `crates/cli/tests/`)

| Case | Setup | Expected |
|---|---|---|
| Token from env var | `GITHUB_TOKEN=abc`, no `--token` | token `abc` is used |
| Token from flag | `--token xyz`, `GITHUB_TOKEN=abc` | token `xyz` is used (flag wins) |
| No token | neither flag nor env var | exits with error message, code `1` |

---

## Constitution Update

Remove from **What It Is Not**:
> A tool for scanning private repos (v1 scope: public repos only)

Add to **What It Is**:
> Capable of scanning private repositories when provided a GitHub token with `repo` scope

---

## Out of Scope

- GitHub App authentication (JWT + installation tokens)
- OAuth device flow
- Token scope pre-validation before scanning begins
- Token rotation or refresh
- Structured output (`--output json`)
- Storing tokens in a local config file
