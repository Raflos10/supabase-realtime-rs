use std::{pin::Pin, time::Duration};

use crate::{
    channel::RealtimeChannelMutableState,
    channel_event::ChannelEvent,
    error::RealtimeError,
    protocol_objects::{Message, Payload},
};

// Constants
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(25);

pub type Result<Type> = std::result::Result<Type, RealtimeError>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChannelState {
    Joined,
    #[default]
    Closed,
    Errored,
    Joining,
    Leaving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscribeState {
    Subscribed,
    TimedOut,
    Closed,
    ChannelError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PushReplyStatus {
    Ok,
    TimedOut,
    Error,
}

pub(crate) type ConnectionMessageReceivedEvent =
    Box<dyn Fn(Message) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
pub(crate) type SubscribeCallback = Box<dyn Fn(Result<SubscribeState>) + Send + Sync>;
pub(crate) type PushCallback = Box<dyn Fn(&Payload) + Send + Sync>;

pub(crate) struct Binding {
    callback: ChannelEvent,
    id: Option<i64>,
}

impl Binding {
    pub(crate) fn new(callback: ChannelEvent, id: Option<i64>) -> Self {
        Self { callback, id }
    }

    pub(crate) fn invoke(
        &self,
        channel_state: &mut RealtimeChannelMutableState,
        payload: Payload,
        _ref: Option<&str>,
        should_remove_channel: &mut bool,
    ) {
        self.callback
            .invoke(channel_state, payload, _ref, should_remove_channel)
    }
}

pub(crate) struct Hook {
    status: PushReplyStatus,
    callback: PushCallback,
}

impl Hook {
    pub(crate) fn new(status: PushReplyStatus, callback: PushCallback) -> Self {
        Self { status, callback }
    }

    pub(crate) fn get_status(&self) -> &PushReplyStatus {
        &self.status
    }

    pub(crate) fn invoke(&self, payload: &Payload) {
        (self.callback)(payload);
    }
}

#[derive(Clone)]
pub(crate) struct PayloadResponse {
    status: PushReplyStatus,
    payload: Payload,
}

impl PayloadResponse {
    pub(crate) fn new(status: PushReplyStatus, payload: Payload) -> Self {
        Self { status, payload }
    }

    pub(crate) fn get_status(&self) -> &PushReplyStatus {
        &self.status
    }

    pub(crate) fn get_payload(&self) -> &Payload {
        &self.payload
    }
}
