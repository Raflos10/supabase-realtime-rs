use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PhxJoin {
    pub config: JoinConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub access_token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JoinConfig {
    #[serde(rename = "broadcast")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcast: Option<BroadcastConfig>,
    #[serde(rename = "presence")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<PresenceConfig>,
    #[serde(rename = "postgres_changes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postgres_changes: Option<Vec<JoinPostgresChanges>>,
    pub private: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BroadcastConfig {
    #[serde(rename = "self")]
    pub self_item: bool,
    pub ack: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PresenceConfig {
    pub key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JoinPostgresChanges {
    pub event: JoinPostgresChangedEvent,
    pub schema: String,
    pub table: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub filter: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum JoinPostgresChangedEvent {
    #[default]
    #[serde(rename = "*")]
    All,
    #[serde(rename = "INSERT")]
    Insert,
    #[serde(rename = "UPDATE")]
    Update,
    #[serde(rename = "DELETE")]
    Delete,
}
