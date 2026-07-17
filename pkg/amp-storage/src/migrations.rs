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
    // v2: User management for multi-tenant BYOK
    "CREATE TABLE IF NOT EXISTS users (
        api_key TEXT PRIMARY KEY,
        user_id TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL DEFAULT '',
        created_at TEXT NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS user_routes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id TEXT NOT NULL REFERENCES users(user_id),
        model TEXT NOT NULL,
        provider TEXT NOT NULL,
        endpoint TEXT NOT NULL,
        auth_header TEXT DEFAULT 'Authorization',
        api_key_encrypted TEXT NOT NULL DEFAULT '',
        created_at TEXT NOT NULL,
        UNIQUE(user_id, model)
    );",
    "CREATE INDEX IF NOT EXISTS idx_user_routes_user ON user_routes(user_id);",
];
