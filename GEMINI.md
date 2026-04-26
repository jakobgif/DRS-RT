# Project Mandates

This file provides foundational instructions for the Gemini CLI agent.

## Priorities & Context

- **Follow CLAUDE.md:** The instructions, workflow, and engineering standards defined in `CLAUDE.md` are authoritative for this project.
- **Hardware Awareness:** Always respect the constraints of the target hardware (Raspberry Pi 4) as defined in `docs/DEVELOPMENT.md`. Avoid optimizations (like spin-waiting) that may cause thermal throttling or conflict with system-level measurement goals.
- **Requirement Traceability:** Reference requirement IDs (e.g., F-1, NF-4, T-2) in all technical discussions, code comments, and commit messages.
