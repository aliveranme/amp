# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

- Repository: **amp** — implement a CLI tool called "amp code" with BYOK (Bring Your Own Key) capabilities.
- Based on analysis of ampcode.com: a frontier coding agent built on Bun/JavaScriptCore, with plugin system, MCP integration, Orbs remote execution, and multi-model (GPT-5.6, Claude Fable 5) support.
- The `main` branch is at `/Volumes/ccc/copilot/amp` (git worktree); this workspace is `aliveranme-init-repo`.
- GitHub: `https://github.com/aliveranme/amp`

## Current Status (2026-07-17)

- **Phase**: Repository initialization and amp code research/RE phase
- **Branch**: `aliveranme-init-repo` (single commit `23ad96b` "Initial commit")
- **Status**: All content staged for first commit; files `A` (staged), `??` (untracked)
- **Main worktree**: `/Volumes/ccc/copilot/amp` on `main` branch

```
Staged:   .gitmodules, resource/utils/unbuned (submodule @ a010095)
Untracked: .claude/, CLAUDE.md, doc/, resource/ (excluding unbuned submodule)
```

All current content is non-code assets:
- Official docs from [ampcode.com](https://ampcode.com/manual) — `doc/manual/`
- Binary reverse engineering artifacts — `resource/`
- Reverse engineering technical docs — `doc/tech/`

## Repository Layout

```
/
├── resource/           # amp CLI binary + RE artifacts
│   ├── amp             # Official amp CLI binary (darwin-arm64, 68MB)
│   ├── install.sh      # Official install script
│   ├── amp-version.txt # Version v0.0.1784219219-g3e9560
│   ├── git-status.txt  # Git status snapshot
│   ├── README.md       # Resource index
│   ├── binary/         # Binary analysis (sections, symbols, strings)
│   │   ├── analysis.md     # Binary overview
│   │   ├── sections/       # Section layout analysis
│   │   ├── symbols/        # Symbol table analysis
│   │   ├── strings/        # Categorized string extractions
│   │   └── extracted/      # JS bundle extracted from binary
│   │       ├── amp.js      # 7MB/6123 lines extracted JS
│   │       └── analysis.md # Extraction analysis
│   └── utils/          # RE tools
│       └── unbuned/    # Bun binary JS extractor (Python 3.6+)
│           └── unbuned.py  # python3 unbuned.py <bun-binary>
│
├── doc/                # Official + RE technical docs
│   ├── README.md       # Doc index
│   ├── OVERVIEW.md     # amp overview & features
│   ├── manual/         # 13 chapters of official docs
│   └── tech/           # 3 RE technical docs
│       ├── architecture.md   # System architecture analysis
│       ├── plugin-system.md  # Plugin system reverse engineering
│       └── protocol.md       # Protocol/OAuth/BYOK analysis
│
├── src/                # (future) Source code for amp code CLI
├── pkg/                # (future) Shared packages
├── cmd/                # (future) CLI entry points
└── CLAUDE.md           # This file
```

## Reverse Engineering Tools

| Tool | Location | Type |
|------|----------|------|
| **unbuned** | `resource/utils/unbuned/unbuned.py` | git submodule — Bun JS extractor |

### unbuned

- **Source**: https://github.com/vibheksoni/unbuned (MIT License)
- **Version**: `v1.0.0-2-ga010095` (pinned submodule commit)
- **Requires**: Python 3.6+, no external dependencies
- **Usage with amp**:
  ```bash
  # From repo root
  python3 resource/utils/unbuned/unbuned.py resource/amp
  # Output goes to ./output/<name>/<name>.js
  cp output/amp/amp.js resource/binary/extracted/
  rm -rf output/
  ```
- **Output**: ~7MB / 6123 lines of minified JavaScript to `output/amp/amp.js`
- **Capabilities**: Mach-O (thin), PE, magic-byte fallback; no ELF/FAT Mach-O support
- **Limitations**: Extracts bundled JS only (no native modules/assets); minified code stays minified

### Submodule Management

```bash
# Initial clone (after cloning this repo)
git submodule init
git submodule update

# Pull latest upstream changes
cd resource/utils/unbuned
git checkout main
git pull
cd ../..
git add resource/utils/unbuned
git commit -m "chore: update unbuned submodule"

# Or one-liner
git submodule update --remote resource/utils/unbuned
git add resource/utils/unbuned
git commit -m "chore: update unbuned submodule"
```

**Important**: The submodule is pinned to a specific commit. Always commit the updated submodule pointer after pulling upstream changes.

## Extracted JS Bundle Structure

From `resource/binary/extracted/amp.js` (7MB):

| Component | Details |
|-----------|---------|
| Runtime | Bun Bake (`// @bun` marker), JSC bytecode compiled |
| CI Build | `/home/runner/work/amp/amp/` |
| Package Manager | pnpm (`.pnpm` directory structure) |
| Key Dependencies | `@napi-rs/keyring@1.1.10` (system keychain), `@grpc/grpc-js@1.14.4`, `pino@9.14.0`, `win-ca@3.5.1` |
| Key Exports | `findCredentials`, `pino`, `transport`, `Ajv`, `RequestHandler` |
| Bun APIs Used | `Bun.spawn`, `Bun.file`, `Bun.plugin`, `Bun.Image`, `bun:ffi` |

## Naming Conventions

- **resource/**: Binary artifacts and RE data only. `resource/binary/` for analysis, `resource/utils/` for RE tools. Sub-directories: `sections/`, `symbols/`, `strings/`, `extracted/`.
- **doc/**: Documentation only. `doc/manual/` for official docs, `doc/tech/` for RE technical docs.
- Directory names are lowercase, hyphen-separated where needed (e.g., `plugin-system.md`).
- Chapter files use zero-padded numbering for ordering (e.g., `01-getting-started.md`).
- Use `README.md` as directory index files.

## Design Principles

The amp code CLI should follow amp's proven architecture patterns:
- **Plugin-first extensibility**: RegisterTool / RegisterCommand event-driven architecture
- **Multi-model BYOK**: Allow users to bring their own API keys for LLMs (OpenAI, Claude, etc.)
- **MCP integration**: Model Context Protocol for tool extensibility (future)
- **Agent modes**: Different levels of capability (low/medium/high/ultra)
- **Thread/session management**: Persistent, shareable agent sessions

## Git conventions

- Worktrees: use git worktrees for isolated development (`.claude/worktrees/` or sibling directories).
- Commit format: conventional commits preferred.
- Push directly: `git push origin HEAD` (no GitHub Actions CI configured yet).
