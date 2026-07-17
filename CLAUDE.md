# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

- Repository: **amp** — implement a CLI tool called "amp code" with BYOK (Bring Your Own Key) capabilities.
- Based on analysis of ampcode.com: a frontier coding agent built on Bun/JavaScriptCore, with plugin system, MCP integration, Orbs remote execution, and multi-model (GPT-5.6, Claude Fable 5) support.
- The `main` branch is at `/Volumes/ccc/copilot/amp` (git worktree); this workspace is `aliveranme-init-repo`.
- GitHub: `https://github.com/aliveranme/amp`

## Current Status (2026-07-17)

- **Phase**: BYOK backend implementation — Rust server with Next.js frontend
- **Branch**: `aliveranme-init-repo` (current worktree), `develop` (integration)
- **Last Commit**: `4e5eac4` — "fix: review findings - add GET user endpoint, apiFetch helper, loading guard"
- **Pushed to GitHub**: `main`, `develop`, `aliveranme-init-repo`
- **License**: MIT

```
main                    → 23ad96b  Initial commit
develop                 → 4e5eac4  (squash-merged from aliveranme-init-repo)
aliveranme-init-repo*   → 4e5eac4  current worktree
```

## Repository Layout

```
/
├── .gitignore           # Excludes: amp binary (68MB), extracted JS (7MB), .claude/, /output/
├── .gitmodules          # Submodule: resource/utils/unbuned
├── CLAUDE.md            # Project guidelines (this file)
│
├── resource/            # amp CLI binary + RE artifacts
│   ├── binary/          # Binary analysis (sections, symbols, strings)
│   └── utils/unbuned/   # git submodule — Bun binary JS extractor
│
├── doc/                 # Official + RE technical docs
│   ├── manual/          # 13 chapters of official docs (from ampcode.com/manual)
│   └── tech/            # RE technical docs + source-map + BYOK design
│
├── cmd/amp-code/        # CLI entry point (Rust, clap)
├── pkg/
│   ├── amp-core/        # Shared types (Thread, Session, Config, Error)
│   ├── amp-proxy/       # Model router + SSE streaming proxy
│   ├── amp-server/      # Axum HTTP server with all routes
│   └── amp-storage/     # SQLite persistence (migrations, CRUD)
│
├── web/                 # Next.js + shadcn/ui admin frontend
│   ├── app/             # Pages (Dashboard, Admin)
│   ├── components/ui/   # shadcn/ui CLI-installed components
│   └── lib/             # API client + types
│
├── route-config.toml    # Default model route table
├── opencode.toml        # OpenCode AI route config
└── Cargo.toml           # Rust workspace root
```

## Build / Test / Run

```bash
# Rust backend
cargo build                 # Compile
cargo check                 # Type-check (fast)
cargo run -- --server       # Start BYOK server (http://localhost:8080)

# Frontend
cd web && npm run dev       # Dev server (http://localhost:3000)
cd web && npm run build     # Static export to web/out/
cd web && npm run lint      # ESLint

# Unified production
cd web && npm run build && cd .. && cargo run -- --server
# Frontend served at / from the Rust backend (static export)
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust (edition 2021), axum 0.8, tokio, sqlx 0.8 (SQLite), reqwest 0.12, clap 4 |
| Frontend | Next.js 16.2 (App Router), React 19, shadcn/ui v4 (base-ui), Tailwind v4 |
| Routing | TOML route config, user-specific routes in DB |
| Auth | API key lookup (`users` table), code-exchange login flow |

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `AMP_API_KEY` | Yes | — | BYOK 服务的用户 API Key（用户身份凭证） |
| `AMP_MODEL_DEFAULT` | No | `gpt-4o` | 默认模型名 |
| `AMP_MODEL_ROUTE` | No | built-in | TOML 路由配置文件路径 |
| `AMP_URL` | No | — | 自定义端点覆盖（路由未匹配时用） |
| `AMP_HOST` | No | `127.0.0.1` | 服务器绑定地址 |
| `AMP_PORT` | No | `8080` | 服务器端口 |
| `AMP_DB_PATH` | No | `amp-code.db` | SQLite 数据库路径 |
| `NEXT_PUBLIC_API_URL` | No | `http://localhost:8080` | 前端 API 地址 |

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | 健康检查 |
| POST | `/auth/token` | 认证（api_key 或 code） |
| GET | `/api/user` | 用户信息 |
| POST | `/v1/chat/completions` | 聊天代理（SSE stream） |
| GET/POST | `/api/threads` | 线程 CRUD |
| GET/POST/DELETE/PATCH | `/admin/api/*` | 管理 API（用户/路由） |

## BYOK 架构

```
amp CLI (AMP_API_KEY=xxx, AMP_URL=http://localhost:8080)
  ──→ BYOK Server (auth → user lookup → route config)
      ──→ Provider API (OpenAI / Anthropic / 自定义)
```

- `AMP_API_KEY` = BYOK 用户身份凭证，对应 `users` 表的 `api_key`
- 每个用户在数据库中有独立的路由规则（模型→Provider 映射）
- 管理员通过 `/admin/api/*` 或前端管理界面配置用户和路由

## Git / PR 流程

- **分支**: `main` ← `develop` ← `feat/*` `fix/*` `chore/*` `docs/*`
- **PR 优先**: 任何改动必须开 PR，自审后 squash merge 到 `develop`
- **Worktree**: 每个分支在独立 git worktree 中开发
- **`gh` CLI**: 优先使用 `gh` 进行 PR/issue 管理

## Reverse Engineering Tools

| Tool | Location | Usage |
|------|----------|-------|
| **unbuned** | `resource/utils/unbuned/unbuned.py` | `python3 resource/utils/unbuned/unbuned.py resource/amp` — extracts JS from Bun binaries |

Submodule management: `git submodule update --remote resource/utils/unbuned`

## Naming Conventions

- `resource/` = binary artifacts and RE data; `doc/` = documentation only
- Directory: lowercase, hyphen-separated; Files: numbered chapters for ordering
- Use `README.md` as directory index files
