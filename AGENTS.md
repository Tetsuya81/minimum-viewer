# Agent Guidelines

<!-- Do not restructure or delete sections. Update individual values in-place when they change. -->

## Core Principles

- **Do NOT maintain backward compatibility** unless explicitly requested. Break things boldly.
- **Keep this file under 20-30 lines of instructions.** Every line competes for the agent's limited context budget (~150-200 total).

---

## Project Overview

**Project type:** Rust TUI (Terminal UI) file viewer/manager inspired by SuperFile and Helix
**Primary language:** Rust (Edition 2021)
**Key dependencies:** `ratatui 0.29`, `crossterm 0.28`

---

## Commands

```bash
# Development
make run          # or: cargo run

# Testing
make test         # or: cargo test

# Build
make build        # or: cargo build

# Environment (Nix/direnv)
direnv allow
```

---

## Code Conventions

- Follow the existing patterns in the codebase
- Prefer explicit over clever
- Delete dead code immediately
- Mode-based state machine: Browse, Filter, Command, Shell, Create
- Command registry pattern via `COMMAND_SPECS` in `src/command/`
- XDG Base Directory Specification for config/state paths
- Tests use dependency injection for environment variables

---

## Architecture

```
src/
├── main.rs              # Entry point, event loop, TUI rendering
├── app.rs               # Core state (Mode, DirEntry, App)
├── ui.rs                # UI rendering
├── config.rs            # XDG config, env vars
├── debug_log.rs         # Debug utilities
└── command/             # Command handlers
    ├── mod.rs           # Registry & dispatcher
    ├── types.rs         # CommandId, CommandSpec
    ├── cd.rs delete.rs editor.rs help.rs
    ├── mkdir.rs path.rs quit.rs rename.rs
```

---

## Maintenance Notes

<!-- This section is permanent. Do not delete. -->

**Keep this file lean and current:**

1. **Remove placeholder sections** (sections still containing `[To be determined]` or `[Add your ... here]`) once you fill them in
2. **Review regularly** - stale instructions poison the agent's context
3. **CRITICAL: Keep total under 20-30 lines** - move detailed docs to separate files and reference them
4. **Update commands immediately** when workflows change
5. **Rewrite Architecture section** when major architectural changes occur
6. **Delete anything the agent can infer** from your code

**Remember:** Coding agents learn from your actual code. Only document what's truly non-obvious or critically important.
