---
name: sdd-refine-spec
description: Use when the user wants to improve, clarify, or fill gaps in an existing spec. Triggers on "refine spec", "update spec", "open questions in spec", or when a spec has TODO/TBD sections or underspecified behaviors.
---

# SDD Refine Spec

## Overview

Investigate open points in an existing spec using research subagents, then discuss findings with the user and update the spec.

## Process

**Step 1 — Load spec**
Read the latest spec from `docs/specs/` (highest number), or the one the user specifies.

**Step 2 — Identify open points**
Scan the spec for:
- Sections marked TODO, TBD, or `???`
- Vague language ("somehow", "as needed", "to be determined")
- Missing error cases or edge conditions
- Unresolved design choices (A vs B decisions)
- Inconsistencies with `docs/constitution.md`, `docs/tech-stack.md`, or `docs/rules.md`

List all open points clearly before dispatching any subagents.

**Step 3 — Spawn investigation subagents**
For each open point, dispatch an `Explore` or `general-purpose` subagent to investigate.
Run independent investigations in parallel (single message, multiple Agent calls).

Each subagent prompt should specify:
- The specific question to answer
- Relevant files or patterns to search
- Whether to search the codebase, web, or both

**Step 4 — Present findings**
Summarize what each subagent found. For each open point, propose a concrete resolution (updated wording, new section, design decision).

**Step 5 — Discuss with user**
Present proposals. Incorporate feedback. Do not write to file yet.

**Step 6 — Update spec**
Edit the spec file with agreed changes. Do not change the `Status` field.

## Rules

- Read the full spec before identifying open points — do not skim
- Dispatch subagents in parallel when investigations are independent
- Do NOT edit the spec until the user has reviewed and agreed to each proposed change
- Do NOT change `Status` from `Draft` to `Approved` — that is the user's decision
- If the user says "investigate X", treat X as an explicit open point regardless of what the spec says
