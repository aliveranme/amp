# BYOK 项目 (amp code CLI) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a BYOK (Bring Your Own Key) CLI tool that proxies LLM requests through a configurable model route engine, with a Rust backend, Next.js + shadcn/ui frontend, and full session management.

**Architecture:** Two-tier system — a Rust backend (Cargo workspace with core/proxy/server/storage crates) providing REST+SSE endpoints, and a Next.js frontend (App Router + shadcn/ui) for configuration and session monitoring. Communication via REST API + SSE streaming.

**Tech Stack:** Rust (axum, tokio, reqwest, clap, serde, sqlx), Next.js (App Router, React 19, TypeScript), shadcn/ui (CLI-installed components), SQLite (via sqlx), SSE for streaming.

## Global Constraints

- All Rust code follows 2024 edition, `rustfmt` default style
- Frontend uses ONLY shadcn/ui components installed via `npx shadcn@latest add` — no custom CSS components
- API response format follows OpenAI-compatible JSON schema for model endpoints
- SSE streaming uses `text/event-stream` content type
- Environment variables: `AMP_API_KEY`, `AMP_URL`, `AMP_MODEL_DEFAULT`, `AMP_MODEL_ROUTE`
- SQLite as primary storage (no PostgreSQL dependency for MVP)
- Every HTTP endpoint has a matching test

---

## File Structure

```
amp-code/
├── Cargo.toml                          # Workspace root
├── cmd/amp-code/
│   ├── Cargo.toml
│   └── src/main.rs                     # CLI entry (clap)
├── pkg/amp-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── thread.rs                   # Thread/Message types
│       ├── session.rs                  # Session types
│       ├── config.rs                   # Configuration types
│       └── error.rs                    # Shared error types
├── pkg/amp-proxy/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── router.rs                   # Route table matching
│       ├── injector.rs                 # API key injection
│       ├── transformer.rs              # Request/response format
│       └── streamer.rs                 # SSE streaming proxy
├── pkg/amp-server/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── routes/
│       │   ├── mod.rs
│       │   ├── chat.rs                 # /v1/chat/completions
│       │   ├── thread.rs              # Thread CRUD
│       │   └── session.rs             # Session management
│       ├── middleware.rs               # Auth/logging middleware
│       └── app.rs                      # axum app setup
├── pkg/amp-storage/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── sqlite.rs                   # SQLite operations
│       └── migrations.rs               # Schema migrations
├── web/
│   ├── package.json
│   ├── next.config.ts
│   ├── tsconfig.json
│   ├── components.json                 # shadcn/ui config
│   ├── app/
│   │   ├── layout.tsx                  # Root layout (shadcn ThemeProvider)
│   │   ├── page.tsx                    # Dashboard
│   │   ├── providers/
│   │   │   └── page.tsx               # Provider config page
│   │   ├── threads/
│   │   │   └── page.tsx               # Thread list
│   │   └── settings/
│   │       └── page.tsx               # Settings page
│   ├── components/
│   │   └── ui/                         # shadcn/ui components (auto-installed)
│   ├── lib/
│   │   ├── api.ts                      # Backend API client
│   │   └── types.ts                    # Shared TypeScript types
│   └── hooks/
│       └── use-threads.ts              # Thread data hook
└── route-config.toml                    # Default model route table
```

---

### Task 1: Rust Workspace Scaffolding + Core Types

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `cmd/amp-code/Cargo.toml`
- Create: `cmd/amp-code/src/main.rs`
- Create: `pkg/amp-core/Cargo.toml`
- Create: `pkg/amp-core/src/lib.rs`
- Create: `pkg/amp-core/src/thread.rs`
- Create: `pkg/amp-core/src/session.rs`
- Create: `pkg/amp-core/src/config.rs`
- Create: `pkg/amp-core/src/error.rs`
- Create: `pkg/amp-proxy/Cargo.toml`
- Create: `pkg/amp-proxy/src/lib.rs`
- Create: `pkg/amp-server/Cargo.toml`
- Create: `pkg/amp-server/src/lib.rs`
- Create: `pkg/amp-storage/Cargo.toml`
- Create: `pkg/amp-storage/src/lib.rs`

**Interfaces:**
- Consumes: nothing (first task)
- Produces: `amp_core::Thread`, `amp_core::Session`, `amp_core::Config`, `amp_core::AppError`, `amp_core::Message`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["cmd/amp-code", "pkg/amp-core", "pkg/amp-proxy", "pkg/amp-server", "pkg/amp-storage"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
reqwest = { version = "0.12", features = ["stream"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
clap = { version = "4", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
toml = "0.8"
thiserror = "2"
```

- [ ] **Step 2: Create pkg/amp-core/src/thread.rs**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreadStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: Uuid,
    pub title: String,
    pub status: ThreadStatus,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}
```

- [ ] **Step 3: Create pkg/amp-core/src/session.rs**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentMode {
    Low,
    Medium,
    High,
    Ultra,
}

impl std::fmt::Display for AgentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentMode::Low => write!(f, "low"),
            AgentMode::Medium => write!(f, "medium"),
            AgentMode::High => write!(f, "high"),
            AgentMode::Ultra => write!(f, "ultra"),
        }
    }
}

impl std::str::FromStr for AgentMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(AgentMode::Low),
            "medium" => Ok(AgentMode::Medium),
            "high" => Ok(AgentMode::High),
            "ultra" => Ok(AgentMode::Ultra),
            _ => Err(format!("Invalid agent mode: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Paused,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub agent_mode: AgentMode,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub context: Option<serde_json::Value>,
}
```

- [ ] **Step 4: Create pkg/amp-core/src/config.rs**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRoute {
    pub provider: String,
    pub endpoint: String,
    pub auth_header: Option<String>,
    pub auth_scheme: Option<String>,
    pub extra_headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    #[serde(rename = "route")]
    pub routes: HashMap<String, ModelRoute>,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub url: Option<String>,
    pub default_model: String,
    pub route_config_path: Option<String>,
    pub host: String,
    pub port: u16,
    pub db_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            url: None,
            default_model: "gpt-4o".to_string(),
            route_config_path: None,
            host: "127.0.0.1".to_string(),
            port: 8080,
            db_path: "amp-code.db".to_string(),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("AMP_API_KEY").unwrap_or_default(),
            url: std::env::var("AMP_URL").ok(),
            default_model: std::env::var("AMP_MODEL_DEFAULT")
                .unwrap_or_else(|_| "gpt-4o".to_string()),
            route_config_path: std::env::var("AMP_MODEL_ROUTE").ok(),
            ..Default::default()
        }
    }
}
```

- [ ] **Step 5: Create pkg/amp-core/src/error.rs**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Storage error: {0}")]
    Storage(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Upstream API error: {0}")]
    Upstream(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::NotFound(_) => (axum::http::StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidRequest(_) => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Config(_) => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Upstream(_) => (axum::http::StatusCode::BAD_GATEWAY, self.to_string()),
            _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, axum::Json(serde_json::json!({"error": message}))).into_response()
    }
}
```

- [ ] **Step 6: Create pkg/amp-core/src/lib.rs**

```rust
pub mod config;
pub mod error;
pub mod session;
pub mod thread;

pub use config::{AppConfig, ModelRoute, RouteConfig};
pub use error::AppError;
pub use session::{AgentMode, Session, SessionStatus};
pub use thread::{Message, MessageRole, Thread, ThreadStatus};
```

- [ ] **Step 7: Create stub Cargo.toml for each crate**

pkg/amp-core/Cargo.toml:
```toml
[package]
name = "amp-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
sqlx = { workspace = true }
axum = { workspace = true }
```

pkg/amp-proxy/Cargo.toml:
```toml
[package]
name = "amp-proxy"
version = "0.1.0"
edition = "2021"

[dependencies]
amp-core = { path = "../amp-core" }
serde = { workspace = true }
serde_json = { workspace = true }
reqwest = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
toml = { workspace = true }
```

pkg/amp-server/Cargo.toml:
```toml
[package]
name = "amp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
amp-core = { path = "../amp-core" }
amp-proxy = { path = "../amp-proxy" }
amp-storage = { path = "../amp-storage" }
axum = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tower-http = { workspace = true }
tracing = { workspace = true }
```

pkg/amp-storage/Cargo.toml:
```toml
[package]
name = "amp-storage"
version = "0.1.0"
edition = "2021"

[dependencies]
amp-core = { path = "../amp-core" }
sqlx = { workspace = true }
tokio = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
```

cmd/amp-code/Cargo.toml:
```toml
[package]
name = "amp-code"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "amp-code"
path = "src/main.rs"

[dependencies]
amp-core = { path = "../../pkg/amp-core" }
amp-server = { path = "../../pkg/amp-server" }
clap = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

- [ ] **Step 8: Create CLI entry point**

cmd/amp-code/src/main.rs:
```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "amp-code", version, about = "BYOK LLM proxy CLI")]
struct Cli {
    /// Start the API server
    #[arg(long)]
    server: bool,

    /// Proxy host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Proxy port
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Database path
    #[arg(long, default_value = "amp-code.db")]
    db: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if cli.server {
        tracing::info!("Starting amp-code server on {}:{}", cli.host, cli.port);
        amp_server::serve(&cli.host, cli.port).await;
    } else {
        println!("amp-code BYOK CLI");
        println!("Run with --server to start the proxy server");
    }
}
```

- [ ] **Step 9: Create stub server lib**

pkg/amp-server/src/lib.rs:
```rust
pub mod app;

pub async fn serve(host: &str, port: u16) {
    let app = app::create().await;
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 10: Create stub app**

pkg/amp-server/src/app.rs:
```rust
use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;

pub async fn create() -> Router {
    Router::new()
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}
```

- [ ] **Step 11: Verify compilation**

```
cd /Volumes/ccc/copilot/copilot-worktrees/amp/aliveranme-musical-enigma
cargo check 2>&1
```

Expected: `Compiling amp-core v0.1.0 ... Compiling amp-code v0.1.0 ... Finished`

- [ ] **Step 12: Commit**

```
git add -A && git commit -m "feat: scaffold Rust workspace with core types"
```

---

### Task 2: Storage Layer — SQLite Schema + Migrations

**Files:**
- Create: `pkg/amp-storage/src/sqlite.rs`
- Create: `pkg/amp-storage/src/migrations.rs`

**Interfaces:**
- Consumes: `amp_core::{Thread, Session, Message, ThreadStatus}`
- Produces: `init_pool(path) -> SqlitePool`, `run_migrations(pool)`

- [ ] **Step 1: Write migrations**

pkg/amp-storage/src/migrations.rs:
```rust
pub const MIGRATIONS: &[&str] = &[
    // v1: Initial schema
    "CREATE TABLE IF NOT EXISTS threads (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL DEFAULT '',
        status TEXT NOT NULL DEFAULT 'Active',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        metadata TEXT
    );",
    "CREATE TABLE IF NOT EXISTS messages (
        id TEXT PRIMARY KEY,
        thread_id TEXT NOT NULL REFERENCES threads(id),
        role TEXT NOT NULL,
        content TEXT NOT NULL,
        timestamp TEXT NOT NULL,
        metadata TEXT,
        FOREIGN KEY (thread_id) REFERENCES threads(id)
    );",
    "CREATE TABLE IF NOT EXISTS sessions (
        id TEXT PRIMARY KEY,
        thread_id TEXT NOT NULL REFERENCES threads(id),
        agent_mode TEXT NOT NULL DEFAULT 'medium',
        status TEXT NOT NULL DEFAULT 'Active',
        started_at TEXT NOT NULL,
        last_heartbeat TEXT NOT NULL,
        ended_at TEXT,
        context TEXT
    );",
    "CREATE INDEX IF NOT EXISTS idx_messages_thread_id ON messages(thread_id);",
    "CREATE INDEX IF NOT EXISTS idx_sessions_thread_id ON sessions(thread_id);",
];
```

- [ ] **Step 2: Write SQLite module**

pkg/amp-storage/src/sqlite.rs:
```rust
use amp_core::{Message, MessageRole, Session, SessionStatus, Thread, ThreadStatus};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use uuid::Uuid;

use super::migrations::MIGRATIONS;

pub async fn init_pool(path: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(path)
        .await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    for (i, migration) in MIGRATIONS.iter().enumerate() {
        sqlx::query(migration).execute(pool).await.map_err(|e| {
            tracing::error!("Migration {i} failed: {e}");
            e
        })?;
    }
    tracing::info!("Database migrations applied ({})", MIGRATIONS.len());
    Ok(())
}

// Thread CRUD
pub async fn create_thread(pool: &SqlitePool, title: &str) -> Result<Thread, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO threads (id, title, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(title).bind("Active").bind(&now).bind(&now)
        .execute(pool).await?;
    Ok(Thread {
        id: Uuid::parse_str(&id).unwrap(),
        title: title.to_string(),
        status: ThreadStatus::Active,
        messages: vec![],
        created_at: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        updated_at: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        metadata: None,
    })
}

pub async fn list_threads(pool: &SqlitePool) -> Result<Vec<Thread>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, Option<String>)>(
        "SELECT id, title, status, created_at, updated_at, metadata FROM threads ORDER BY updated_at DESC"
    ).fetch_all(pool).await?;

    let mut threads = Vec::new();
    for (id, title, status, created_at, updated_at, metadata) in rows {
        let status = match status.as_str() {
            "Active" => ThreadStatus::Active,
            _ => ThreadStatus::Archived,
        };
        threads.push(Thread {
            id: Uuid::parse_str(&id).unwrap(),
            title,
            status,
            messages: vec![],
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at).unwrap().into(),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at).unwrap().into(),
            metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
        });
    }
    Ok(threads)
}

// Add a message to a thread
pub async fn add_message(
    pool: &SqlitePool,
    thread_id: &Uuid,
    role: MessageRole,
    content: &str,
) -> Result<Message, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let role_str = match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
        MessageRole::Tool => "tool",
    };
    sqlx::query("INSERT INTO messages (id, thread_id, role, content, timestamp) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(thread_id.to_string()).bind(role_str).bind(content).bind(&now)
        .execute(pool).await?;
    // Update thread updated_at
    sqlx::query("UPDATE threads SET updated_at = ? WHERE id = ?")
        .bind(&now).bind(thread_id.to_string())
        .execute(pool).await?;
    Ok(Message {
        id: Uuid::parse_str(&id).unwrap(),
        role,
        content: content.to_string(),
        timestamp: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        metadata: None,
    })
}

// Session CRUD
pub async fn create_session(
    pool: &SqlitePool,
    thread_id: &Uuid,
    agent_mode: &str,
) -> Result<Session, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, thread_id, agent_mode, status, started_at, last_heartbeat) VALUES (?, ?, ?, ?, ?, ?)"
    ).bind(&id).bind(thread_id.to_string()).bind(agent_mode).bind("Active").bind(&now).bind(&now)
    .execute(pool).await?;
    Ok(Session {
        id: Uuid::parse_str(&id).unwrap(),
        thread_id: *thread_id,
        agent_mode: agent_mode.parse().unwrap_or(amp_core::AgentMode::Medium),
        status: SessionStatus::Active,
        started_at: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        last_heartbeat: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        ended_at: None,
        context: None,
    })
}
```

- [ ] **Step 3: Update pkg/amp-storage/src/lib.rs**

```rust
pub mod migrations;
pub mod sqlite;

pub use sqlite::*;
```

- [ ] **Step 4: Compile and commit**

```bash
cargo check 2>&1
git add -A && git commit -m "feat: add storage layer with SQLite migrations and CRUD"
```

---

### Task 3: Proxy Engine — Route Table + API Key Injection

**Files:**
- Create: `route-config.toml`
- Create: `pkg/amp-proxy/src/router.rs`
- Create: `pkg/amp-proxy/src/injector.rs`

**Interfaces:**
- Consumes: `amp_core::{ModelRoute, RouteConfig, AppConfig}`
- Produces: `Router::from_config(path)`, `Router::route(model) -> &ModelRoute`, `Injector::inject(&ModelRoute, api_key) -> HeaderMap`

- [ ] **Step 1: Default route config**

route-config.toml:
```toml
[route."gpt-4o"]
provider = "openai"
endpoint = "https://api.openai.com/v1/chat/completions"
auth_header = "Authorization"
auth_scheme = "Bearer"

[route."gpt-4o-mini"]
provider = "openai"
endpoint = "https://api.openai.com/v1/chat/completions"
auth_header = "Authorization"
auth_scheme = "Bearer"

[route."claude-sonnet-4"]
provider = "anthropic"
endpoint = "https://api.anthropic.com/v1/messages"
auth_header = "x-api-key"
[route."claude-sonnet-4".extra_headers]
anthropic-version = "2023-06-01"

[route."claude-fable-5"]
provider = "anthropic"
endpoint = "https://api.anthropic.com/v1/messages"
auth_header = "x-api-key"
[route."claude-fable-5".extra_headers]
anthropic-version = "2023-06-01"

[route."*"]
provider = "openai"
endpoint = "https://api.openai.com/v1/chat/completions"
auth_header = "Authorization"
auth_scheme = "Bearer"
```

- [ ] **Step 2: Route router module**

pkg/amp-proxy/src/router.rs:
```rust
use std::collections::HashMap;
use std::path::Path;

use amp_core::ModelRoute;

#[derive(Debug)]
pub struct Router {
    routes: HashMap<String, ModelRoute>,
    fallback: ModelRoute,
}

impl Router {
    pub fn from_config(path: impl AsRef<Path>) -> Result<Self, crate::ProxyError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| crate::ProxyError::Config(format!("Cannot read route config: {e}")))?;
        let config: HashMap<String, ModelRoute> = toml::from_str(&content)
            .map_err(|e| crate::ProxyError::Config(format!("Invalid route config: {e}")))?;
        Self::from_hashmap(config)
    }

    pub fn from_hashmap(mut routes: HashMap<String, ModelRoute>) -> Result<Self, crate::ProxyError> {
        let fallback = routes.remove("*").ok_or_else(|| {
            crate::ProxyError::Config("Route config must have a '*' fallback route".to_string())
        })?;
        Ok(Self { routes, fallback })
    }

    pub fn route(&self, model: &str) -> &ModelRoute {
        self.routes.get(model).unwrap_or(&self.fallback)
    }
}
```

- [ ] **Step 3: API key injector**

pkg/amp-proxy/src/injector.rs:
```rust
use std::collections::HashMap;

use amp_core::ModelRoute;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub fn build_headers(route: &ModelRoute, api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    // Auth header
    if let Some(header_name) = &route.auth_header {
        let value = match &route.auth_scheme {
            Some(scheme) => format!("{scheme} {api_key}"),
            None => api_key.to_string(),
        };
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(header_name.as_bytes()),
            HeaderValue::from_str(&value),
        ) {
            headers.insert(name, val);
        }
    }

    // Extra headers
    if let Some(extra) = &route.extra_headers {
        for (k, v) in extra {
            let resolved = v.replace("${API_KEY}", api_key);
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(&resolved),
            ) {
                headers.insert(name, val);
            }
        }
    }

    headers
}
```

- [ ] **Step 4: Create ProxyError and update lib**

pkg/amp-proxy/src/lib.rs:
```rust
pub mod injector;
pub mod router;

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Config error: {0}")]
    Config(String),
    #[error("Upstream error: {0}")]
    Upstream(String),
}
```

- [ ] **Step 5: Compile and commit**

```bash
cargo check
git add -A && git commit -m "feat: add model route table and API key injector"
```

---

### Task 4: Proxy Engine — SSE Streaming + Request Transformation

**Files:**
- Create: `pkg/amp-proxy/src/streamer.rs`
- Create: `pkg/amp-proxy/src/transformer.rs`

**Interfaces:**
- Consumes: `amp_core::ModelRoute`, `Router`, `Injector`
- Produces: `ProxySession::new(client, route, api_key)`, `stream_chat_completion(body) -> Receiver<ChatChunk>`, `ChatRequest`, `ChatChunk`

- [ ] **Step 1: Chat request/response types**

pkg/amp-proxy/src/transformer.rs:
```rust
use serde::{Deserialize, Serialize};

/// OpenAI-compatible chat completion request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// SSE stream chunk (OpenAI-compatible)
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: DeltaContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DeltaContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}
```

- [ ] **Step 2: SSE streamer proxy**

pkg/amp-proxy/src/streamer.rs:
```rust
use std::pin::Pin;

use futures::Stream;
use reqwest::Client;
use tokio::sync::mpsc;

use amp_core::ModelRoute;

use super::injector::build_headers;
use super::transformer::{ChatChunk, ChatRequest};

pub fn stream_chat_completion(
    client: Client,
    route: ModelRoute,
    api_key: String,
    api_url: Option<String>,
    request: ChatRequest,
) -> impl Stream<Item = Result<ChatChunk, super::ProxyError>> + Send + 'static {
    let (tx, rx) = mpsc::channel::<Result<ChatChunk, super::ProxyError>>(64);

    tokio::spawn(async move {
        let endpoint = api_url.unwrap_or(route.endpoint.clone());
        let mut headers = build_headers(&route, &api_key);
        headers.insert("content-type", "application/json".parse().unwrap());

        let req_body = serde_json::to_value(&request).unwrap();
        let http_request = client
            .post(&endpoint)
            .headers(headers)
            .json(&req_body)
            .send()
            .await;

        match http_request {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    let _ = tx
                        .send(Err(super::ProxyError::Upstream(format!("HTTP {status}: {body}"))))
                        .await;
                    return;
                }

                let mut stream = resp.bytes_stream();
                use futures::StreamExt;
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(&bytes);
                            for line in text.lines() {
                                let line = line.trim();
                                if line.is_empty() || line == "data: [DONE]" {
                                    continue;
                                }
                                if let Some(data) = line.strip_prefix("data: ") {
                                    if let Ok(chunk) = serde_json::from_str::<ChatChunk>(data) {
                                        let _ = tx.send(Ok(chunk)).await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx
                                .send(Err(super::ProxyError::Upstream(e.to_string())))
                                .await;
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                let _ = tx
                    .send(Err(super::ProxyError::Upstream(e.to_string())))
                    .await;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}
```

- [ ] **Step 3: Update pkg/amp-proxy/Cargo.toml dependencies**

Edit `pkg/amp-proxy/Cargo.toml` — add `futures`, `tokio-stream`:
```toml
futures = "0.3"
tokio-stream = "0.1"
```

- [ ] **Step 4: Compile and commit**

```bash
cargo check
git add -A && git commit -m "feat: add SSE streaming proxy and request transformers"
```

---

### Task 5: Server Routes — Chat Proxy + Thread/Session Management

**Files:**
- Create: `pkg/amp-server/src/routes/mod.rs`
- Create: `pkg/amp-server/src/routes/chat.rs`
- Create: `pkg/amp-server/src/routes/thread.rs`
- Create: `pkg/amp-server/src/routes/session.rs`
- Modify: `pkg/amp-server/src/app.rs`

**Interfaces:**
- Consumes: `ProxySession`, `amp_storage::*`, `AppConfig`
- Produces: `POST /v1/chat/completions` (SSE streaming), `GET/POST/PUT /api/threads`, `POST /api/sessions`

- [ ] **Step 1: Chat route**

pkg/amp-server/src/routes/chat.rs:
```rust
use std::sync::Arc;

use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use reqwest::Client;
use tokio_stream::StreamExt;

use amp_core::AppConfig;
use amp_proxy::router::Router;
use amp_proxy::transformer::ChatRequest;

use super::AppState;

pub async fn chat_completion(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let model = if req.model.is_empty() {
        &state.config.default_model
    } else {
        &req.model
    };
    let route = state.router.route(model).clone();

    let stream = amp_proxy::streamer::stream_chat_completion(
        Client::new(),
        route,
        state.config.api_key.clone(),
        state.config.url.clone(),
        req,
    );

    let event_stream = stream.map(|result| match result {
        Ok(chunk) => {
            let json = serde_json::to_string(&chunk).unwrap_or_default();
            Ok(Event::default().data(format!("data: {json}\n\n")))
        }
        Err(e) => Ok(Event::default().data(format!("data: {e}\n\n"))),
    });

    Sse::new(event_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("data: [DONE]\n\n"),
    )
}
```

- [ ] **Step 2: Thread routes**

pkg/amp-server/src/routes/thread.rs:
```rust
use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Deserialize;

use amp_core::Thread;
use amp_storage;

use super::AppState;

#[derive(Deserialize)]
pub struct CreateThreadRequest {
    pub title: Option<String>,
}

pub async fn list_threads(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Thread>> {
    let threads = amp_storage::sqlite::list_threads(&state.pool).await.unwrap_or_default();
    Json(threads)
}

pub async fn create_thread(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateThreadRequest>,
) -> Json<Thread> {
    let title = req.title.unwrap_or_default();
    let thread = amp_storage::sqlite::create_thread(&state.pool, &title)
        .await
        .expect("Failed to create thread");
    Json(thread)
}
```

- [ ] **Step 3: Session routes**

pkg/amp-server/src/routes/session.rs:
```rust
use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Deserialize;
use uuid::Uuid;

use amp_core::Session;
use amp_storage;

use super::AppState;

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub thread_id: String,
    pub agent_mode: Option<String>,
}

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSessionRequest>,
) -> Json<Session> {
    let thread_id = Uuid::parse_str(&req.thread_id).expect("Invalid thread_id");
    let mode = req.agent_mode.as_deref().unwrap_or("medium");
    let session = amp_storage::sqlite::create_session(&state.pool, &thread_id, mode)
        .await
        .expect("Failed to create session");
    Json(session)
}
```

- [ ] **Step 4: Routes mod and AppState**

pkg/amp-server/src/routes/mod.rs:
```rust
pub mod chat;
pub mod session;
pub mod thread;

use sqlx::sqlite::SqlitePool;

use amp_core::AppConfig;
use amp_proxy::router::Router;

pub struct AppState {
    pub config: AppConfig,
    pub router: Router,
    pub pool: SqlitePool,
}
```

- [ ] **Step 5: Update app.rs with all routes**

pkg/amp-server/src/app.rs:
```rust
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

use amp_core::AppConfig;
use amp_proxy::router::Router as AmpRouter;
use amp_storage;

use super::routes::{chat, session, thread, AppState};

pub async fn create(config: AppConfig) -> Router {
    // Init storage
    let pool = amp_storage::sqlite::init_pool(&config.db_path)
        .await
        .expect("Failed to initialize database");

    // Load route config
    let route_router = if let Some(path) = &config.route_config_path {
        AmpRouter::from_config(path).expect("Invalid route config")
    } else {
        let routes = toml::from_str(include_str!("../../../route-config.toml"))
            .expect("Invalid default route config");
        AmpRouter::from_hashmap(routes).expect("Invalid default route config")
    };

    let state = Arc::new(AppState {
        config,
        router: route_router,
        pool,
    });

    Router::new()
        .route("/health", get(health))
        .route("/v1/chat/completions", post(chat::chat_completion))
        .route("/api/threads", get(thread::list_threads))
        .route("/api/threads", post(thread::create_thread))
        .route("/api/sessions", post(session::create_session))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}
```

- [ ] **Step 6: Update server lib.rs**

pkg/amp-server/src/lib.rs:
```rust
pub mod app;
pub mod routes;

pub async fn serve(config: amp_core::AppConfig) {
    let app = app::create(config).await;
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 7: Update CLI entry to pass config**

cmd/amp-code/src/main.rs (rewrite the server block):
```rust
use amp_core::AppConfig;

if cli.server {
    tracing::info!("Starting amp-code server on {}:{}", cli.host, cli.port);
    let mut config = AppConfig::from_env();
    config.host = cli.host;
    config.port = cli.port;
    config.db_path = cli.db;
    amp_server::serve(config).await;
}
```

- [ ] **Step 8: Compile and commit**

```bash
cargo check
git add -A && git commit -m "feat: add chat proxy, thread, and session API routes"
```

---

### Task 6: Frontend — Next.js + shadcn/ui Scaffolding

**Files:**
- Create: `web/package.json`
- Create: `web/next.config.ts`
- Create: `web/tsconfig.json`
- Create: `web/components.json` (shadcn/ui config)
- Create: `web/tailwind.config.ts`
- Create: `web/postcss.config.js`
- Create: `web/app/globals.css`
- Create: `web/app/layout.tsx`
- Create: `web/app/page.tsx`
- Create: `web/lib/types.ts`
- Create: `web/lib/api.ts`

**Interfaces:**
- Consumes: Rust backend at `http://localhost:8080`
- Produces: Dashboard page with thread list, settings

- [ ] **Step 1: Initialize Next.js project**

```bash
mkdir -p web && cd web
npx create-next-app@latest . --typescript --tailwind --eslint --app --src-dir=false --import-alias="@/*" --use-npm 2>&1 || true
```

If interactive prompts appear, answer: Yes to TypeScript, Yes to Tailwind, Yes to App Router.

- [ ] **Step 2: Install shadcn/ui**

```bash
cd web
npx shadcn@latest init -d --force 2>&1 || true
```

- [ ] **Step 3: Install core shadcn/ui components**

```bash
cd web
npx shadcn@latest add button card table dialog input label badge separator theme \
  --yes 2>&1 || true
```

- [ ] **Step 4: Frontend types**

web/lib/types.ts:
```typescript
export interface Thread {
  id: string;
  title: string;
  status: 'Active' | 'Archived';
  messages: Message[];
  created_at: string;
  updated_at: string;
}

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  timestamp: string;
}

export interface Session {
  id: string;
  thread_id: string;
  agent_mode: 'low' | 'medium' | 'high' | 'ultra';
  status: 'Active' | 'Paused' | 'Ended';
  started_at: string;
  last_heartbeat: string;
}
```

- [ ] **Step 5: API client**

web/lib/api.ts:
```typescript
const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export async function fetchThreads(): Promise<Thread[]> {
  const res = await fetch(`${API_BASE}/api/threads`);
  return res.json();
}

export async function createThread(title: string): Promise<Thread> {
  const res = await fetch(`${API_BASE}/api/threads`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ title }),
  });
  return res.json();
}
```

- [ ] **Step 6: Root layout with shadcn theme**

web/app/layout.tsx:
```tsx
import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "amp code BYOK",
  description: "BYOK LLM Proxy Manager",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="zh-CN" suppressHydrationWarning>
      <body className={`${inter.className} antialiased`}>
        {children}
      </body>
    </html>
  );
}
```

- [ ] **Step 7: Dashboard page**

web/app/page.tsx:
```tsx
"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { fetchThreads, createThread } from "@/lib/api";
import type { Thread } from "@/lib/types";

export default function Dashboard() {
  const [threads, setThreads] = useState<Thread[]>([]);
  const [title, setTitle] = useState("");

  useEffect(() => {
    fetchThreads().then(setThreads).catch(console.error);
  }, []);

  const handleCreate = async () => {
    if (!title.trim()) return;
    const newThread = await createThread(title);
    setThreads([newThread, ...threads]);
    setTitle("");
  };

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b">
        <div className="container mx-auto px-4 py-4 flex items-center justify-between">
          <h1 className="text-xl font-bold">amp code BYOK</h1>
          <Badge variant="outline">Proxy Active</Badge>
        </div>
      </header>

      <main className="container mx-auto px-4 py-8">
        <Card className="mb-8">
          <CardHeader>
            <CardTitle>New Thread</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Input
              placeholder="Thread title..."
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleCreate()}
            />
            <Button onClick={handleCreate}>Create</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Threads</CardTitle>
          </CardHeader>
          <CardContent>
            {threads.length === 0 ? (
              <p className="text-muted-foreground">No threads yet.</p>
            ) : (
              <div className="space-y-2">
                {threads.map((t) => (
                  <div key={t.id}>
                    <div className="flex items-center justify-between py-2">
                      <span className="font-medium">{t.title || "Untitled"}</span>
                      <Badge>{t.status}</Badge>
                    </div>
                    <Separator />
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </main>
    </div>
  );
}
```

- [ ] **Step 8: Commit**

```bash
git add -A && git commit -m "feat: scaffold Next.js frontend with shadcn/ui dashboard"
```

---

### Task 7: Integration — End-to-End Verification

**Files:**
- Modify: `cmd/amp-code/src/main.rs` (add `--serve` default behavior)
- Create: `tests/integration_test.rs` (if test directory doesn't exist)

- [ ] **Step 1: Quickstart the server**

```bash
cargo run -- --server --port 8090
```

Expected: `Listening on 127.0.0.1:8090`

- [ ] **Step 2: Test health endpoint**

```bash
curl -s http://127.0.0.1:8090/health | jq .
```

Expected: `{"status":"ok"}`

- [ ] **Step 3: Test thread API**

```bash
curl -s -X POST http://127.0.0.1:8090/api/threads -H 'Content-Type: application/json' -d '{"title":"test thread"}' | jq .
```

Expected: Thread object with id, title, status

- [ ] **Step 4: Test frontend**

```bash
cd web && npm run dev
```

Open `http://localhost:3000` — dashboard should load with thread list.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: integration - server starts, threads CRUD, frontend connects"
```
