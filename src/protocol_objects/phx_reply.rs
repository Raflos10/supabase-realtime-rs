use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", content = "response")]
pub enum PhxReply {
    #[serde(rename = "error")]
    Error(ErrorReply),
    #[serde(rename = "ok")]
    Ok(PhxResponse),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorReply {
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhxResponse {
    #[serde(default)]
    #[serde(rename = "postgres_changes")]
    pub postgres_changes: Vec<ReplyPostgresChanges>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplyPostgresChanges {
    pub event: ReplyPostgresChangedEvent,
    pub schema: String,
    pub table: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub filter: Option<String>,
    pub id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReplyPostgresChangedEvent {
    #[serde(rename = "*")]
    All,
}
