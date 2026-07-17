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
