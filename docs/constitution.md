# Constitution

## Mission

`repo-scanner` is a CLI tool that helps developers verify that a GitHub repository is safe **before** cloning or running any of its code.

It exists because malicious actors use fake hiring processes and other social engineering tactics to trick developers into running repositories that contain malware. The tool gives developers a fast, trustworthy answer: is this repo safe to open?

## What It Is

- A static analysis tool — it reads code, it never executes it
- A pre-clone safety gate — the primary use case is "should I clone this repo?"
- A rule-driven scanner — all detection logic is expressed as explicit, human-readable rules
- A composable CLI tool — exits with a meaningful code so it can be used in shell scripts

## What It Is Not

- A runtime sandbox or firewall
- A replacement for antivirus or EDR tooling
- A general-purpose SAST tool (though it may overlap)
- A tool for scanning private repos (v1 scope: public repos only)

## Core Principles

**Transparency over magic.** Every finding must trace back to a specific, named rule with a human-readable description. Developers should always be able to understand why something was flagged.

**Safe by default.** The tool never executes repository code. GitHub API access is read-only. The `--clone` mode downloads to a temp directory but never runs anything.

**Extensibility through rules.** Adding support for a new threat or a new language should not require changes to core infrastructure — only adding a new rule entry or a new analyzer implementation.

**Fail loudly on configuration errors.** An invalid ruleset is caught at startup, not silently ignored at scan time.

**Exit codes are part of the contract.** `0` (safe), `1` (suspicious), `2` (dangerous) are stable and documented. They will not change without a major version bump.

## Success Criteria

A developer can run `repo-scanner https://github.com/owner/repo` and within seconds receive a clear verdict — Safe, Suspicious, or Dangerous — with a list of specific findings (file, line, rule, severity) that explains the verdict.
