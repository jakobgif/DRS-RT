# DRS Round-Trip Measurement

See @README.md for project overview.

# Requirements

@docs/REQUIREMENTS.md

# Development & Deployment

@docs/DEVELOPMENT.md

# Performance

This project measures round-trip latency. **Performance is the top priority.** Every decision — data structures, algorithms, system calls, memory layout, I/O placement — must be evaluated for its impact on latency. Always ask: does this add latency to the hot path? If yes, find an alternative or move it out of the measurement loop.

# Implementation

When implementing any feature, always reference the requirement number(s) being targeted (e.g., F-1, NF-4, T-2) in code comments, commit messages, and responses.

When writing Rust, prefer established, well-documented crates over hand-rolled solutions where it makes sense. Do not reinvent the wheel. Examples: use `clap` for CLI parsing, `log`/`env_logger` for logging. Only reach for a crate if it is widely used and actively maintained.

# Workflow

Use subagents for isolated or verbose tasks (codebase exploration, log analysis, research) to keep the main session context lean. Subagents run in their own context window and return only a summary — preventing large tool outputs from polluting the main conversation. Chain them from the main session; subagents cannot spawn other subagents.

For difficult or multi-step tasks, always create a plan before starting. Number each phase (Phase 1, Phase 2, …) and define clear goals per phase. If the plan is too long to fit cleanly in the session, write it to a markdown file in `docs/` and reference it.

# Git Style

Commit proactively after each plan phase is complete, tested, and confirmed working — do not wait to be asked. Follow the git style: one file (or tightly related file group) per commit. Never bundle unrelated files into a single commit. Keep commit messages short and focused — one change, one message. Do not create long commit message chains.

Do not touch already-committed code unless the task requires it. No reformatting, no comment tweaks, no whitespace cleanup as a side effect. Before committing, review the full diff and remove any unintended changes.

Never commit "current state" snapshots or plan markdown files. These exist as working files only and must not enter git history. Before every commit, review what is staged and exclude any planning or status documents.

Always add Claude as co-author in every commit message:

```
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
```

# Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

# Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.
