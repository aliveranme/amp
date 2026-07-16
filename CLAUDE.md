# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

- Repository: **amp** — implement a CLI tool called "amp code" with BYOK (Bring Your Own Key) capabilities.
- Based on analysis of ampcode.com: a frontier coding agent built on Bun/JavaScriptCore, with plugin system, MCP integration, Orbs remote execution, and multi-model (GPT-5.6, Claude Fable 5) support.
- The `main` branch is at `/Volumes/ccc/copilot/amp` (git worktree); this workspace is `aliveranme-init-repo`.
- GitHub: `https://github.com/aliveranme/amp`

## Current Status (2026-07-17)

- **Phase**: Source code analysis completed — full module map and BYOK design written
- **Branch**: `aliveranme-init-repo` (current worktree), `develop` (integration)
- **Last Commit**: `161c1e9` — "docs: fix layout tree, update status, add BYOK proxy protocol"
- **Pushed to GitHub**: `main`, `develop`, `aliveranme-init-repo`
- **License**: MIT

```
main                    → 23ad96b  Initial commit
develop                 → ea7f600  docs: fix layout tree, update status, add BYOK proxy
aliveranme-init-repo*   → 161c1e9  docs: fix layout tree, update status, add BYOK proxy
```

## Repository Layout

```
/
├── .gitignore           # Excludes: amp binary (68MB), extracted JS (7MB), .claude/, /output/
├── .gitmodules          # Submodule: resource/utils/unbuned
├── CLAUDE.md            # Project guidelines (this file)
│
├── resource/            # amp CLI binary + RE artifacts
│   ├── amp              # (gitignored) Official amp CLI binary (darwin-arm64, 68MB)
│   ├── install.sh       # Official install script
│   ├── amp-version.txt  # Version v0.0.1784219219-g3e9560
│   ├── git-status.txt   # Git status snapshot
│   ├── README.md        # Resource index
│   ├── binary/          # Binary analysis (sections, symbols, strings)
│   │   ├── analysis.md      # Binary overview
│   │   ├── sections/        # Section layout analysis
│   │   ├── symbols/         # Symbol table analysis
│   │   ├── strings/         # Categorized string extractions (131,867 total)
│   │   └── extracted/       # JS bundle extracted from binary
│   │       ├── amp.js       # (gitignored) 7MB/6123 lines extracted JS
│   │       └── analysis.md  # Extraction analysis
│   └── utils/           # RE tools
│       └── unbuned/     # git submodule — Bun binary JS extractor (MIT)
│
├── doc/                 # Official + RE technical docs
│   ├── README.md        # Doc index
│   ├── OVERVIEW.md      # amp overview & features
│   ├── manual/          # 13 chapters of official docs (from ampcode.com/manual)
│   └── tech/            # RE technical docs
│       ├── architecture.md   # System architecture analysis
│       ├── plugin-system.md  # Plugin system reverse engineering
│       ├── protocol.md       # Protocol/OAuth/BYOK/proxy/env analysis
│       └── source-map/       # Source code module mapping
│           ├── module-map.md     # Full module decomposition
│           └── byok-design.md    # BYOK implementation design
│
├── src/                 # (future) Source code for amp code CLI
├── pkg/                 # (future) Shared packages
├── cmd/                 # (future) CLI entry points
└── CLAUDE.md            # This file
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

## BYOK 接口约定

amp code CLI 的核心 BYOK 接口：

```env
AMP_API_KEY=<provider-api-key>   # 必需：LLM 提供商的 API 密钥
AMP_URL=<https://instance-url>    # 可选：自定义后端地址
```

- `AMP_API_KEY` — 透传到目标 LLM Provider 的认证密钥（OpenAI / Anthropic 等）
- `AMP_URL` — 自定义部署地址，路由未匹配时转发到此端点
- 模型路由逻辑完全由 CLI 本地完成，后端只需兼容 OpenAI API 格式

## License

- MIT License — see root `LICENSE` file (to be created with first source code commit)

## CI / Automation

- **GitHub Actions**: Standard CI workflows to be added with source code. Initial scope:
  - `build` — 编译验证
  - `test` — 单元测试 / 集成测试
  - `lint` — 代码格式化检查
- **PR merge**: 自审后可 squash merge。标准 Actions 自动运行后绿通过即可合并
- **Secrets**: `AMP_API_KEY` etc. stored as GitHub Actions secrets for integration tests

## Git conventions

- **Branching strategy**: 每项功能/改动必须在独立分支上开发，通过 PR 合并回来
  - `main` — 稳定分支，只接受从 `develop` 来的 PR
  - `develop` — 集成分支，所有 feature 分支 PR 到此
  - `feat/<name>` — 新功能分支
  - `fix/<name>` — 修复分支
  - `chore/<name>` — 杂务分支
  - `docs/<name>` — 文档分支
- **PR 优先**: 任何改动都必须创建 PR，经过 review 后合并。禁止直接 push 到 `main` 或 `develop`
- **Worktrees**: 每个分支在独立 git worktree 中开发（`/Volumes/ccc/copilot/<branch-name>/`）
- **Commit 格式**: conventional commits（`feat:`、`fix:`、`docs:`、`chore:` 等）
- **合并策略**: feature branch → PR → squash merge 到 `develop` → 最终 PR → merge 到 `main`

## GitHub / Git 管理

优先使用 `gh` CLI 管理仓库和 GitHub 操作：

```bash
# 创建 worktree 新分支
cd /Volumes/ccc/copilot
git worktree add -b feat/xxx ../amp-feat-xxx develop

# PR 流程（在 worktree 分支上）
git add -A && git commit -m "feat: ..."
git push -u origin feat/xxx
gh pr create --base develop --title "feat: ..." --body "Changes: ..."
gh pr review --approve   # 自审或请求 review
gh pr merge --squash     # squash merge

# 清理
cd /Volumes/ccc/copilot
git worktree remove ../amp-feat-xxx
git branch -d feat/xxx        # 本地删除
git push origin --delete feat/xxx  # 远程删除

# 同步 develop
git branch -D develop          # 删除本地 stale
git fetch origin develop
git worktree add -b develop ../amp-develop origin/develop

# 其他常用 gh 命令
gh pr list                     # 查看 PR
gh pr checkout <number>        # 切换到 PR
gh issue list                  # 查看 issues
gh repo view                   # 查看仓库
```
