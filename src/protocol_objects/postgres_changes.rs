use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PostgresDataChangeEvent {
    Insert,
    Update,
    Delete,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PostgresChangesData {
    pub columns: Vec<Column>,
    pub commit_timestamp: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
    #[serde(default)]
    pub old_record: Option<Value>,
    #[serde(default)]
    pub record: Option<Value>,
    pub schema: String,
    pub table: String,
    #[serde(rename = "type")]
    pub type_: PostgresDataChangeEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostgresChangesPayload {
    pub data: PostgresChangesData,
    pub ids: Vec<i64>,
}
