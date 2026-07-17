use amp_core::{AgentMode, Message, MessageRole, Session, SessionStatus, Thread, ThreadStatus};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

use super::migrations::MIGRATIONS;

// ---------------------------------------------------------------------------
// Pool init
// ---------------------------------------------------------------------------

pub async fn init_pool(path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = if path.starts_with("sqlite://") {
        SqliteConnectOptions::from_str(path)?
    } else if path == ":memory:" {
        SqliteConnectOptions::from_str("sqlite::memory:")?
    } else {
        // Resolve relative paths against current directory
        let abs_path = if Path::new(path).is_absolute() {
            path.to_string()
        } else {
            let cwd = std::env::current_dir().unwrap_or_default();
            cwd.join(path).to_string_lossy().to_string()
        };
        SqliteConnectOptions::from_str(&format!("sqlite://{}", abs_path))?
    }
    .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

pub(crate) async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    for (i, migration) in MIGRATIONS.iter().enumerate() {
        if let Err(e) = sqlx::query(migration).execute(pool).await {
            let msg = e.to_string();
            // SQLite ALTER TABLE ADD COLUMN throws "duplicate column" if column exists
            if msg.contains("duplicate column name") {
                tracing::warn!("Migration {i}: column exists, skipping: {msg}");
            } else {
                tracing::error!("Migration {i} failed: {e}");
                return Err(e);
            }
        }
    }
    tracing::info!("Database migrations applied ({})", MIGRATIONS.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// Thread CRUD
// ---------------------------------------------------------------------------

pub async fn create_thread(pool: &SqlitePool, title: &str) -> Result<Thread, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO threads (id, title, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(title).bind("Active").bind(&now).bind(&now)
        .execute(pool).await?;
    Ok(Thread {
        id: uuid::Uuid::parse_str(&id).unwrap(),
        title: title.to_string(),
        status: ThreadStatus::Active,
        messages: vec![],
        created_at: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        updated_at: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        metadata: None,
    })
}

pub async fn get_thread(pool: &SqlitePool, id: &uuid::Uuid) -> Result<Option<Thread>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, Option<String>)>(
        "SELECT id, title, status, created_at, updated_at, metadata FROM threads WHERE id = ?"
    )
    .bind(id.to_string())
    .fetch_optional(pool).await?;

    match row {
        None => Ok(None),
        Some((id, title, status, created_at, updated_at, metadata)) => {
            let status = match status.as_str() {
                "Active" => ThreadStatus::Active,
                _ => ThreadStatus::Archived,
            };
            Ok(Some(Thread {
                id: uuid::Uuid::parse_str(&id).unwrap(),
                title,
                status,
                messages: vec![],
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at).unwrap().into(),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at).unwrap().into(),
                metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
            }))
        }
    }
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
            id: uuid::Uuid::parse_str(&id).unwrap(),
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

pub async fn update_thread_status(
    pool: &SqlitePool,
    id: &uuid::Uuid,
    status: ThreadStatus,
) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let status_str = match status {
        ThreadStatus::Active => "Active",
        ThreadStatus::Archived => "Archived",
    };
    let rows = sqlx::query("UPDATE threads SET status = ?, updated_at = ? WHERE id = ?")
        .bind(status_str)
        .bind(&now)
        .bind(id.to_string())
        .execute(pool).await?
        .rows_affected();
    Ok(rows > 0)
}

pub async fn delete_thread(pool: &SqlitePool, id: &uuid::Uuid) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM sessions WHERE thread_id = ?")
        .bind(id.to_string())
        .execute(&mut *tx).await?;
    sqlx::query("DELETE FROM messages WHERE thread_id = ?")
        .bind(id.to_string())
        .execute(&mut *tx).await?;
    let rows = sqlx::query("DELETE FROM threads WHERE id = ?")
        .bind(id.to_string())
        .execute(&mut *tx).await?
        .rows_affected();
    tx.commit().await?;
    Ok(rows > 0)
}

// ---------------------------------------------------------------------------
// Message CRUD
// ---------------------------------------------------------------------------

pub async fn add_message(
    pool: &SqlitePool,
    thread_id: &uuid::Uuid,
    role: MessageRole,
    content: &str,
) -> Result<Message, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let role_str = match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
        MessageRole::Tool => "tool",
    };
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO messages (id, thread_id, role, content, timestamp) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(thread_id.to_string()).bind(role_str).bind(content).bind(&now)
        .execute(&mut *tx).await?;
    sqlx::query("UPDATE threads SET updated_at = ? WHERE id = ?")
        .bind(&now).bind(thread_id.to_string())
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Message {
        id: uuid::Uuid::parse_str(&id).unwrap(),
        role,
        content: content.to_string(),
        timestamp: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        metadata: None,
    })
}

pub async fn list_messages(
    pool: &SqlitePool,
    thread_id: &uuid::Uuid,
) -> Result<Vec<Message>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT id, role, content, timestamp, metadata FROM messages WHERE thread_id = ? ORDER BY timestamp ASC"
    )
    .bind(thread_id.to_string())
    .fetch_all(pool).await?;

    let mut messages = Vec::new();
    for (id, role_str, content, timestamp, metadata) in rows {
        let role = match role_str.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            "tool" => MessageRole::Tool,
            _ => continue,
        };
        messages.push(Message {
            id: uuid::Uuid::parse_str(&id).unwrap(),
            role,
            content,
            timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp).unwrap().into(),
            metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
        });
    }
    Ok(messages)
}

// ---------------------------------------------------------------------------
// Session CRUD
// ---------------------------------------------------------------------------

pub async fn create_session(
    pool: &SqlitePool,
    thread_id: &uuid::Uuid,
    agent_mode: &str,
) -> Result<Session, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, thread_id, agent_mode, status, started_at, last_heartbeat) VALUES (?, ?, ?, ?, ?, ?)"
    ).bind(&id).bind(thread_id.to_string()).bind(agent_mode).bind("Active").bind(&now).bind(&now)
    .execute(pool).await?;
    Ok(Session {
        id: uuid::Uuid::parse_str(&id).unwrap(),
        thread_id: *thread_id,
        agent_mode: agent_mode.parse().unwrap_or(AgentMode::Medium),
        status: SessionStatus::Active,
        started_at: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        last_heartbeat: chrono::DateTime::parse_from_rfc3339(&now).unwrap().into(),
        ended_at: None,
        context: None,
    })
}

pub async fn get_session(
    pool: &SqlitePool,
    id: &uuid::Uuid,
) -> Result<Option<Session>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<String>)>(
        "SELECT id, thread_id, agent_mode, status, started_at, last_heartbeat, ended_at, context FROM sessions WHERE id = ?"
    )
    .bind(id.to_string())
    .fetch_optional(pool).await?;

    match row {
        None => Ok(None),
        Some((id, thread_id, agent_mode, status, started_at, last_heartbeat, ended_at, context)) => {
            let status_enum = match status.as_str() {
                "Active" => SessionStatus::Active,
                "Paused" => SessionStatus::Paused,
                _ => SessionStatus::Ended,
            };
            Ok(Some(Session {
                id: uuid::Uuid::parse_str(&id).unwrap(),
                thread_id: uuid::Uuid::parse_str(&thread_id).unwrap(),
                agent_mode: agent_mode.parse().unwrap_or(AgentMode::Medium),
                status: status_enum,
                started_at: chrono::DateTime::parse_from_rfc3339(&started_at).unwrap().into(),
                last_heartbeat: chrono::DateTime::parse_from_rfc3339(&last_heartbeat).unwrap().into(),
                ended_at: ended_at.and_then(|e| chrono::DateTime::parse_from_rfc3339(&e).ok().map(Into::into)),
                context: context.and_then(|c| serde_json::from_str(&c).ok()),
            }))
        }
    }
}

pub async fn list_sessions(
    pool: &SqlitePool,
    thread_id: &uuid::Uuid,
) -> Result<Vec<Session>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<String>)>(
        "SELECT id, thread_id, agent_mode, status, started_at, last_heartbeat, ended_at, context FROM sessions WHERE thread_id = ? ORDER BY started_at DESC"
    )
    .bind(thread_id.to_string())
    .fetch_all(pool).await?;

    let mut sessions = Vec::new();
    for (id, tid, agent_mode, status, started_at, last_heartbeat, ended_at, context) in rows {
        let status_enum = match status.as_str() {
            "Active" => SessionStatus::Active,
            "Paused" => SessionStatus::Paused,
            _ => SessionStatus::Ended,
        };
        sessions.push(Session {
            id: uuid::Uuid::parse_str(&id).unwrap(),
            thread_id: uuid::Uuid::parse_str(&tid).unwrap(),
            agent_mode: agent_mode.parse().unwrap_or(AgentMode::Medium),
            status: status_enum,
            started_at: chrono::DateTime::parse_from_rfc3339(&started_at).unwrap().into(),
            last_heartbeat: chrono::DateTime::parse_from_rfc3339(&last_heartbeat).unwrap().into(),
            ended_at: ended_at.and_then(|e| chrono::DateTime::parse_from_rfc3339(&e).ok().map(Into::into)),
            context: context.and_then(|c| serde_json::from_str(&c).ok()),
        });
    }
    Ok(sessions)
}

pub async fn update_session_heartbeat(pool: &SqlitePool, id: &uuid::Uuid) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows = sqlx::query("UPDATE sessions SET last_heartbeat = ? WHERE id = ?")
        .bind(&now)
        .bind(id.to_string())
        .execute(pool).await?
        .rows_affected();
    Ok(rows > 0)
}

pub async fn end_session(pool: &SqlitePool, id: &uuid::Uuid) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows = sqlx::query("UPDATE sessions SET status = 'Ended', ended_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id.to_string())
        .execute(pool).await?
        .rows_affected();
    Ok(rows > 0)
}
