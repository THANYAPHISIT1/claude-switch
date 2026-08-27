# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo check                          # fast syntax/type check
cargo clippy -- -D warnings          # linter — must be clean before install
cargo test                           # run all unit tests
cargo test <test_name>               # run a single test
cargo build --release                # production build (no dashboard)
make install                         # build + install to Homebrew Cellar + re-link
make uninstall                       # remove from Homebrew Cellar entirely
```

### Dashboard (dev)

```bash
# Run dashboard locally against a profile's session history
cargo run --features dashboard -- dashboard --<profile>
# e.g.
cargo run --features dashboard -- dashboard --personal

# Build + install with dashboard feature
make install-dashboard               # bumps version check same as make install
```

Dashboard opens at `http://localhost:4040` by default. Ctrl+C to stop.

`make install` / `make install-dashboard` reads the version from `Cargo.toml` automatically and will refuse to install if the same version is already linked — bump the version first.

Distribution is via a public Homebrew tap (`brew tap THANYAPHISIT1/tap`).

## Architecture

The binary is split into a **library crate** (`src/lib.rs`) and a thin **binary entry point** (`src/main.rs` ~15 lines). `main.rs` calls `parse_args()` → either `prompt::run_interactive_mode()` (TUI) or `run_switch(args)` (direct).

### Request flow

```
main() → parse_args()
           ├── Version/Help → parser::print_version() / print_help()
           ├── Interactive → prompt::run_interactive_mode() → CliArgs
           └── Direct(CliArgs) ─────────────────────────────────┐
                                                                 ▼
                                                         run_switch(CliArgs)
                                                           ├── Profile::new()       (security/token.rs)
                                                           └── runner::run_claude() (proxy/runner.rs)
```

`runner::run_claude()` spawns the `claude` process with `CLAUDE_CONFIG_DIR=~/.claude-<profile>` — this is what isolates histories, MCP servers, settings, and keychain tokens per profile.

The reserved profile `default` is the exception: it maps to Claude Code's own `~/.claude`, and `run_claude()` **unsets** `CLAUDE_CONFIG_DIR` (rather than setting it) so the child behaves identically to running `claude` bare. Setting it explicitly would derive a different keychain entry, and an inherited value from the parent shell would silently override the profile. Because of this, `default` needs no alias and cannot be deleted from the Manage menu.

### Keychain — how profile isolation works

Claude Code derives its keychain service name from `CLAUDE_CONFIG_DIR`:

```
"Claude Code-credentials-" + SHA256(config_dir_path)[0..8]
```

Example:
- `~/.claude-work`     → `"Claude Code-credentials-XXXXXXXX"`
- `~/.claude-personal` → `"Claude Code-credentials-YYYYYYYY"`

The hash is taken over the literal `CLAUDE_CONFIG_DIR` string — symlinks are **not** resolved, so pointing it at a symlink yields a different (empty) keychain entry. The default `~/.claude` uses the unsuffixed `"Claude Code-credentials"` when `CLAUDE_CONFIG_DIR` is unset.

Because each profile has a unique `CLAUDE_CONFIG_DIR`, Claude Code automatically reads and writes a **separate keychain entry per profile**. No manual keychain manipulation is needed — do not add restore/backup logic that deletes and recreates keychain entries, as it breaks the ACL that grants Claude Code access to the entry.

### TUI (`src/cli/prompt.rs`)

Uses `dialoguer` + `console`. Profile status (● saved / ○ new) is determined by whether `~/.claude-<profile>/` is non-empty (i.e., Claude has been used in that profile before). The Manage submenu handles: Delete, Reveal in Finder, Set alias, Remove alias.

Alias blocks written to `~/.zshrc` use a marker comment `# claude-switch: <profile> profile` to identify and overwrite/remove them cleanly. Each alias block includes a base alias and a `-fast` variant (`--dangerously-skip-permissions`).

### Platform stubs

`security/win_cred.rs`, `security/mac_keychain.rs`, and `commands/mcp_sync.rs` are empty stubs.
