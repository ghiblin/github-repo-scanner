---
name: sdd-implement-plan
description: Use when the user wants to execute an implementation plan. Triggers on "implement plan", "execute plan", "start implementation", "implement the latest plan", or a reference to a specific plan number.
---

# SDD Implement Plan

## Overview

Load the target plan and delegate execution to the appropriate superpowers skill.

## Process

**Step 1 — Load plan**
- If the user specifies a plan (by name or number), read that file from `docs/plans/`
- Otherwise, read the latest plan (highest numbered file in `docs/plans/`)

**Step 2 — Invoke execution skill**
Use **`superpowers:subagent-driven-development`** (preferred — runs tasks in parallel via isolated worktrees).
Fall back to `superpowers:executing-plans` for sequential execution if worktrees are not appropriate.

**Step 3 — Follow the plan exactly**
- Each task has `- [ ]` checkboxes — mark each complete as work finishes
- Do NOT skip TDD steps (failing test → implement → passing test → commit)
- Do NOT combine tasks or reorder them unless the user explicitly approves

## Rules

- REQUIRED: invoke `superpowers:subagent-driven-development` before starting implementation
- Do NOT start implementing until the plan is loaded and the execution skill is invoked
- Do NOT mark a task complete until: tests pass AND commit is done
- If a task's tests cannot be made to pass, stop and report to the user — do not skip ahead
- If the user says "implement task N", start from that task (do not re-run earlier completed tasks)
