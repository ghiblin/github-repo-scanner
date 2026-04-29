# Tech Stack

## Language

**Rust (stable, latest)** — chosen for memory safety, zero-cost abstractions, and strong type system that enforces correctness at compile time.

## Project Structure

**Cargo workspace** with five crates:

| Crate | Type | Purpose |
|---|---|---|
| `cli` | binary | Entry point, argument parsing, wires up all contexts |
| `repository` | library | Fetches repo content via GitHub API or local clone |
| `rules` | library | Loads and validates TOML rule files |
| `analysis` | library | Runs language-specific analyzers, produces findings |
| `report` | library | Formats and prints terminal output |

Library crates are pure domain logic — no I/O side effects except `repository`. Only `cli` and `repository` are async.

## Dependencies

### CLI
- **`clap`** (derive feature) — argument parsing

### Networking
- **`reqwest`** (with `json` feature) — HTTP client for GitHub API calls
- **`tokio`** (`rt-multi-thread`, `macros`) — async runtime and `#[tokio::main]` / `#[tokio::test]` macros

### Serialization
- **`uuid`** (`v7`, `serde`) — UUID v7 generation for Value Object identifiers
- **`serde`** (derive feature) — serialization framework
- **`toml`** — TOML rule file parsing
- **`serde_json`** — GitHub API response parsing

### Error Handling
- **`thiserror`** — domain error enums in library crates
- **`anyhow`** — user-facing error formatting in `cli` crate only

### Terminal Output
- **`colored`** — ANSI color output for the report
- **`indicatif`** — progress bar during API fetch

### Testing
- **`mockito`** — HTTP mock server for `repository` crate tests
- **`insta`** — snapshot testing for `report` crate
- **`tempfile`** — temporary directories for `--clone` mode tests

## Tooling

- **`clippy`** with `pedantic` — linting, enforced in CI
- **`rustfmt`** — code formatting, enforced in CI
- **`cargo test`** — unit and integration tests
- **`cargo nextest`** — faster test runner (optional, for local dev)

## Minimum Rust Version

**MSRV: 1.75** (required for async traits via `async_fn_in_trait` stabilization)

## No Build Scripts

No `build.rs` files. No proc macros beyond `derive`. The build must be simple and auditable.
