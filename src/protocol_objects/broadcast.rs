use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Broadcast {
    #[serde(rename = "type")]
    pub r#type: String,
    pub event: String,
}

// TODO: payload