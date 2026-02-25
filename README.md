# 🔄 Claude-Switch

**A blazingly fast, stateless profile and environment manager for Claude Code, written in Rust. 🦀**

Claude-Switch acts as a transparent proxy for Anthropic's Claude Code CLI. It solves the pain point of session conflicts and environment bleeding by providing true OS-level isolation for different workspaces and API providers.

Whether you are juggling frontend projects (Nuxt, Next, Svelte) in your `--personal` profile, managing backend systems (Node.js, Python, Go) in your `--work` profile, or experimenting with LLMs via Langchain and local APIs, Claude-Switch keeps your contexts strictly separated.

## ✨ Key Features

* **OS-Level Keychain Isolation:** Seamlessly backs up and restores OAuth tokens using the macOS `security` keychain. No more being forced to re-login when switching contexts.
* **Stateless Environments:** Dynamically routes `CLAUDE_CONFIG_DIR` to dedicated folders (e.g., `~/.claude-work`, `~/.claude-personal`). Chat histories, MCP servers, and logs never mix.
* **Custom API Provider Support:** Easily run third-party models (like Z.ai / GLM) by isolating `settings.json` within specific profiles, bypassing default Anthropic authentication.
* **Pass-Through Architecture:** 100% compatible with standard Claude Code flags and `npx` commands.
* **Zero Overhead:** Compiled to a single native binary for instant execution.

## 🚀 Installation

### For Users (Via Homebrew)
If you just want to use the tool, you can install the pre-built binary via my public tap:

```bash
brew tap THANYAPHISIT1/tap
brew install claude-switch
```

### For Developers (Build from source)

If you want to modify the code or build it locally:

```bash
git clone [https://github.com/THANYAPHISIT1/claude-switch.git](https://github.com/THANYAPHISIT1/claude-switch.git)
cd claude-switch
cargo install --path .
```

## 💻 Usage

Simply prepend your desired profile flag before any Claude Code command.

**Basic Profile Switching:**

```bash
claude-switch --work
claude-switch --personal
claude-switch --glm
```

**Using with MCP Servers & Plugins:**
Claude-Switch passes all arguments directly to Claude Code. You can install MCP servers to specific profiles effortlessly:

```bash
# Install GitHub MCP only to the 'work' profile
claude-switch --work mcp add github -- npx -y @modelcontextprotocol/server-github
```

## 🧠 How it Works (Under the Hood)

When you run `claude-switch --<profile>`, the Rust binary performs the following lifecycle:

1. **Intercept & Route:** Detects the profile flag and sets `CLAUDE_CONFIG_DIR=~/.claude-<profile>`.
2. **Keychain Swap (Pre-run):** Extracts the profile's specific OAuth token from the macOS keychain and injects it into the active session.
3. **Execution:** Spawns a child process to run the actual `claude` CLI with the remaining arguments.
4. **State Backup (Post-run):** Once the session ends, it captures any new authentication tokens and securely saves them back to the macOS keychain under the profile's specific namespace.

## ⚠️ Requirements

* Currently optimized for **macOS** (relies on the native `security` command for keychain management).
* Node.js and [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) must be installed globally.

## 🗺️ Roadmap

* [ ] Add interactive TUI (Terminal User Interface) for profile selection.
* [ ] Implement automated CI/CD for Homebrew binary distribution.
* [ ] Windows Credential Manager / Linux Secret Service support.
