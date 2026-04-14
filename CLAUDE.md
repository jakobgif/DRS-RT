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

# Workflow

Use subagents for isolated or verbose tasks (codebase exploration, log analysis, research) to keep the main session context lean. Subagents run in their own context window and return only a summary — preventing large tool outputs from polluting the main conversation. Chain them from the main session; subagents cannot spawn other subagents.
