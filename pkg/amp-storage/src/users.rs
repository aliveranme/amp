use sqlx::sqlite::SqlitePool;

use serde::Serialize;

/// Create a new user with a random API key.
pub async fn create_user(
    pool: &SqlitePool,
    name: &str,
) -> Result<UserRow, sqlx::Error> {
    let user_id = uuid::Uuid::new_v4().to_string();
    let api_key = format!("amp_byok_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO users (api_key, user_id, name, created_at) VALUES (?, ?, ?, ?)")
        .bind(&api_key)
        .bind(&user_id)
        .bind(name)
        .bind(&now)
        .execute(pool)
        .await?;

    Ok(UserRow { api_key, user_id, name: name.to_string(), created_at: now })
}

/// Look up a user by API key. Returns Ok(None) if not found, Err on DB failure.
pub async fn find_user_by_key(pool: &SqlitePool, api_key: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT api_key, user_id, name, created_at FROM users WHERE api_key = ?"
    )
    .bind(api_key)
    .fetch_optional(pool)
    .await
}

/// List all users.
pub async fn list_users(pool: &SqlitePool) -> Result<Vec<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>("SELECT api_key, user_id, name, created_at FROM users ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

/// Get a single user by user_id.
pub async fn get_user_by_id(pool: &SqlitePool, user_id: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT api_key, user_id, name, created_at FROM users WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Get total user count.
pub async fn user_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
}

/// Get total route count.
pub async fn route_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM user_routes")
        .fetch_one(pool)
        .await
}

/// Update user name.
pub async fn update_user_name(pool: &SqlitePool, user_id: &str, name: &str) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("UPDATE users SET name = ? WHERE user_id = ?")
        .bind(name).bind(user_id)
        .execute(pool).await?;
    Ok(r.rows_affected() > 0)
}

/// Delete a user by user_id.
pub async fn delete_user(pool: &SqlitePool, user_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ─── Routes ───────────────────────────────────────────────────────

/// Get a user's route configuration.
pub async fn get_user_routes(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<UserRouteRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRouteRow>(
        "SELECT id, user_id, model, provider, endpoint, auth_header, api_key_encrypted, created_at
         FROM user_routes WHERE user_id = ? ORDER BY model"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Find a route for a specific user + model. Falls back to '*' wildcard.
pub async fn find_user_route(
    pool: &SqlitePool,
    user_id: &str,
    model: &str,
) -> Result<Option<UserRouteRow>, sqlx::Error> {
    let exact = sqlx::query_as::<_, UserRouteRow>(
        "SELECT id, user_id, model, provider, endpoint, auth_header, api_key_encrypted, created_at
         FROM user_routes WHERE user_id = ? AND model = ?"
    )
    .bind(user_id).bind(model)
    .fetch_optional(pool).await?;

    if exact.is_some() {
        return Ok(exact);
    }

    // Fallback to wildcard
    sqlx::query_as::<_, UserRouteRow>(
        "SELECT id, user_id, model, provider, endpoint, auth_header, api_key_encrypted, created_at
         FROM user_routes WHERE user_id = ? AND model = '*'"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Add or update a route for a user.
pub async fn upsert_user_route(
    pool: &SqlitePool,
    user_id: &str,
    model: &str,
    provider: &str,
    endpoint: &str,
    api_key: &str,
    auth_header: &str,
    enabled: bool,
    rate_limit: i64,
    max_tokens: i64,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO user_routes (user_id, model, provider, endpoint, auth_header, api_key_encrypted, created_at, enabled, rate_limit, max_tokens)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id, model) DO UPDATE SET
           provider = excluded.provider, endpoint = excluded.endpoint,
           auth_header = excluded.auth_header, api_key_encrypted = excluded.api_key_encrypted,
           enabled = excluded.enabled, rate_limit = excluded.rate_limit, max_tokens = excluded.max_tokens"
    )
    .bind(user_id).bind(model).bind(provider).bind(endpoint)
    .bind(auth_header).bind(api_key).bind(&now)
    .bind(enabled as i32).bind(rate_limit).bind(max_tokens)
    .execute(pool).await?;
    Ok(())
}

/// Delete a user route.
pub async fn delete_user_route(
    pool: &SqlitePool,
    user_id: &str,
    model: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_routes WHERE user_id = ? AND model = ?")
        .bind(user_id).bind(model)
        .execute(pool).await?;
    Ok(result.rows_affected() > 0)
}

/// Toggle route enabled/disabled.
pub async fn set_route_enabled(
    pool: &SqlitePool,
    user_id: &str,
    model: &str,
    enabled: bool,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("UPDATE user_routes SET enabled = ? WHERE user_id = ? AND model = ?")
        .bind(enabled as i32).bind(user_id).bind(model)
        .execute(pool).await?;
    Ok(r.rows_affected() > 0)
}

// ─── Usage Tracking ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct UsageEntry {
    pub model: String,
    pub provider: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub duration_ms: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub total_requests: i64,
    pub total_tokens_in: i64,
    pub total_tokens_out: i64,
    pub by_model: Vec<ModelUsage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelUsage {
    pub model: String,
    pub requests: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
}

/// Log a usage entry after a proxy request.
pub async fn log_usage(
    pool: &SqlitePool,
    user_id: &str,
    model: &str,
    provider: &str,
    tokens_in: i64,
    tokens_out: i64,
    duration_ms: i64,
    status: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO usage_logs (user_id, model, provider, tokens_in, tokens_out, duration_ms, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(user_id).bind(model).bind(provider)
    .bind(tokens_in).bind(tokens_out).bind(duration_ms).bind(status).bind(&now)
    .execute(pool).await?;
    Ok(())
}

/// Get usage summary for a user (last N days).
pub async fn get_user_usage(
    pool: &SqlitePool,
    user_id: &str,
    days: i32,
) -> Result<UsageSummary, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, i64, i64, i64)>(
        "SELECT model, COUNT(*) as requests, SUM(tokens_in) as tokens_in, SUM(tokens_out) as tokens_out
         FROM usage_logs WHERE user_id = ? AND created_at > datetime('now', ?)
         GROUP BY model ORDER BY requests DESC"
    )
    .bind(user_id).bind(format!("-{days} days"))
    .fetch_all(pool).await?;

    let mut total_req = 0i64;
    let mut total_in = 0i64;
    let mut total_out = 0i64;
    let mut by_model = Vec::new();

    for (model, req, tin, tout) in rows {
        total_req += req;
        total_in += tin;
        total_out += tout;
        by_model.push(ModelUsage { model, requests: req, tokens_in: tin, tokens_out: tout });
    }

    Ok(UsageSummary {
        total_requests: total_req,
        total_tokens_in: total_in,
        total_tokens_out: total_out,
        by_model,
    })
}

/// Get global usage summary (for admin dashboard).
pub async fn get_global_usage(
    pool: &SqlitePool,
    days: i32,
) -> Result<UsageSummary, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, i64, i64, i64)>(
        "SELECT model, COUNT(*) as requests, SUM(tokens_in) as tokens_in, SUM(tokens_out) as tokens_out
         FROM usage_logs WHERE created_at > datetime('now', ?)
         GROUP BY model ORDER BY requests DESC"
    )
    .bind(format!("-{days} days"))
    .fetch_all(pool).await?;

    let mut total_req = 0i64;
    let mut total_in = 0i64;
    let mut total_out = 0i64;
    let mut by_model = Vec::new();

    for (model, req, tin, tout) in rows {
        total_req += req;
        total_in += tin;
        total_out += tout;
        by_model.push(ModelUsage { model, requests: req, tokens_in: tin, tokens_out: tout });
    }

    Ok(UsageSummary {
        total_requests: total_req,
        total_tokens_in: total_in,
        total_tokens_out: total_out,
        by_model,
    })
}

// ─── Row types ──────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct UserRow {
    pub api_key: String,
    pub user_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow, Clone, Serialize)]
pub struct UserRouteRow {
    pub id: i64,
    pub user_id: String,
    pub model: String,
    pub provider: String,
    pub endpoint: String,
    pub auth_header: String,
    pub api_key_encrypted: String,
    pub created_at: String,
    pub enabled: bool,
    pub rate_limit: i64,
    pub max_tokens: i64,
}
