use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PresenceState(pub HashMap<String, Presence>);

#[derive(Clone, Serialize, Deserialize)]
pub struct Presence {
    pub metas: Vec<PresenceMeta>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PresenceMeta {
    pub phx_ref: String,
    pub name: Option<String>,
    // TODO: t
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PresenceDiff {
    pub joins: HashMap<String, Presence>,
    pub leaves: HashMap<String, Presence>,
}
