# Pokedex: High-Performance Agentic Swarm Engine (Rust) 🚀

**Pokedex** is a production-ready, clean-room Rust reimplementation of a multi-agent swarm architecture. Originally conceived as a behavioral clone, it has evolved into a **Live Multi-Provider Intelligence System** capable of executing autonomous technical missions using Gemini 2.0 and GitHub Copilot.

---

### ⚖️ Legal & Clean-Room Status

This repository is a **clean-room Rust reimplementation**.
1. **Specification Phase**: An AI agent analyzed behavioral specs to produce exhaustive tool contracts and data flows.
2. **Implementation Phase**: A separate agent implemented the idiomatic Rust core from the spec alone, without ever referencing proprietary source. 
*This mirrors the legal precedent of Phoenix Technologies v. IBM (1984) and Baker v. Selden (1879) — protecting behavior and ideas via clean-room engineering.*

---

## 🌪️ The Engine: Multi-Provider Swarm

Pokedex is no longer a simulation. It is a "Hot" engine that routes agentic intent through industry-leading LLMs to perform real work in your environment.

### 🧠 Multi-Provider Intelligence
- **Gemini 2.0 Flash**: Native integration with Google's `streamGenerateContent` and real-time function calling.
- **GitHub Copilot**: Leverages OpenAI-compatible tool-calls via the GitHub Models API for robust engineering logic.
- **Unified ProviderClient**: A provider-agnostic abstraction layer that normalizes streaming, tool-use, and thinking blocks across different models.

### 🛠️ Agentic Missions & Tools
Agents in the Pokedex swarm are equipped with a high-fidelity toolbelt:
- **Autonomous Execution**: Agents use `BashTool`, `FileEdit`, and `Grep` to solve complex tasks.
- **Containerized Sandboxing**: Commands are executed in isolated **WASM containers** (Kali Linux for security, Alpine for dev) to ensure your host remains secure.
- **MCP Integration**: Full support for Model Context Protocol servers to extend agent capabilities dynamically.

### 🧬 Integrated Persona Libraries
Pokedex ships with two legendary agentic core libraries (located in `/library/`):
- **Agency-Agents**: A suite of specialized personas (Senior SWE, Pentester, UX Researcher, Psychologist) for composite missions.
- **Impeccable**: Advanced skills for system hardening, layout optimization, and design system generation.

---

## ⚡ Quick Start

### 1. Configure Credentials
Pokedex supports multi-key load balancing and dynamic model discovery. Export your keys (comma or semicolon separated):

```bash
# Multi-key Gemini support with round-robin rotation
export GOOGLE_API_KEYS="key1;key2;key3"
# GitHub Models / Copilot tokens
export GITHUB_TOKENS="token1,token2"
```

### 2. Advanced Configuration (Zero-Hardcoding)
The Pokedex Swarm Engine is fully configurable at runtime via environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `SWARM_MAX_CONCURRENCY` | Maximum concurrent agents in a mission | 10 |
| `LLM_MAX_RETRIES` | Retries per model before fallback | 4 |
| `LLM_RETRY_DELAY` | Baseline delay (seconds) between retries | 2 |
| `BROWSER_WAIT_MS` | Headless browser wait time for JS rendering | 1500 |
| `BROWSER_MAX_LEN` | Character limit for captured web content | 120k |
| `POKEDEX_AGENT_LIBRARY` | Subdirectory name for persona markdown files | `agency-agents` |

### 3. Project Scoping & Memory
Pokedex now organizes missions into individual project directories under `Projects/`.
- **Workspace**: Isolated project-local work directory.
- **Swarm Persistence**: Continuous state saving to `Swarm/swarm_state.json`.
- **MemPalace Integration**: Semantic memory is stored in a project-local SQLite database, allowing agents to share long-term context across runs.

### 4. Launch a Mission
Initiate an autonomous swarm mission directly from the CLI:

```bash
# Example: Design and security audit
pokedex --mission "Create a high-conversion user app and then perform a security audit on it"
```


## 📂 Project Structure

- `crates/api`: Unified ProviderClient with Gemini & Copilot support.
- `crates/core`: Configuration, Credential Governance, and Error handling.
- `crates/swarm`: The Brain. Orchestration and persona loading from `agency-agents`.
- `crates/tools`: Containerized tool implementations (WASM/Bash/LSP).
- `crates/tui`: Fast, Ratatui-based terminal interface.

