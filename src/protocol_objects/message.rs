use serde::{Deserialize, Serialize};

use super::payload::Payload;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub topic: String,
    #[serde(flatten)]
    pub payload: Payload,
    #[serde(default)]
    // #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ref")]
    pub ref_field: Option<String>,
}
