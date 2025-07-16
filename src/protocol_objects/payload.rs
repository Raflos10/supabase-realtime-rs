use serde::{Deserialize, Serialize};

use super::{
    access_token::AccessToken,
    broadcast::Broadcast,
    heartbeat::Heartbeat,
    phx_close::PhxClose,
    phx_error::PhxError,
    phx_join::PhxJoin,
    phx_reply::PhxReply,
    postgres_changes::PostgresChangesPayload,
    presence::{PresenceDiff, PresenceState},
    system::System,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload", rename_all = "snake_case")]
pub enum Payload {
    #[serde(rename = "phx_join")]
    PhxJoin(PhxJoin),
    #[serde(rename = "phx_reply")]
    PhxReply(PhxReply),
    #[serde(rename = "phx_close")]
    PhxClose(PhxClose),
    #[serde(rename = "phx_error")]
    PhxError(PhxError),
    #[serde(rename = "heartbeat")]
    Heartbeat(Heartbeat),
    #[serde(rename = "access_token")]
    AccessToken(AccessToken),
    #[serde(rename = "broadcast")]
    Broadcast(Broadcast),
    #[serde(rename = "presence_state")]
    PresenceState(PresenceState),
    #[serde(rename = "presence_diff")]
    PresenceDiff(PresenceDiff),
    #[serde(rename = "system")]
    System(System),
    #[serde(rename = "postgres_changes")]
    PostgresChanges(PostgresChangesPayload),
}
