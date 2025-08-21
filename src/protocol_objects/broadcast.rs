use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Broadcast {
    // #[serde(rename = "type")]
    // pub r#type: String,
    pub event: String,
    pub payload: Value,
}
