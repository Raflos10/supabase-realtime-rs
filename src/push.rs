use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, oneshot::Receiver},
    task::AbortHandle,
};

use crate::{
    client::RealtimeClient,
    protocol_objects::{Message, Payload},
    types::{DEFAULT_TIMEOUT, Hook, PayloadResponse, PushCallback, PushReplyStatus, Result},
};

#[derive(Clone)]
pub struct Push {
    event: String,
    payload: Payload,
    is_sent: bool,
    timeout: Duration,
    _ref: Option<String>,
    received_response: Arc<Mutex<Option<PayloadResponse>>>,
    rec_hooks: Arc<Mutex<Vec<Hook>>>,
    ref_event: Option<String>,
    timeout_abort_handle: Option<AbortHandle>, // TODO: make JoinHandle
}

impl Push {
    pub(crate) fn new(event: &str, payload: Payload, timeout: Option<Duration>) -> Self {
        Self {
            event: String::from(event),
            payload,
            timeout: timeout.unwrap_or(DEFAULT_TIMEOUT),
            _ref: None,
            ref_event: None,
            received_response: Arc::new(Mutex::new(None)),
            is_sent: false,
            timeout_abort_handle: None,
            rec_hooks: Arc::new(Mutex::new(vec![])),
        }
    }

    pub(crate) async fn resend(
        &mut self,
        client: &RealtimeClient,
        topic: &str,
        _ref: &str,
        current_event: &str,
        receiver: Receiver<PayloadResponse>,
    ) -> Result<()> {
        // self.cancel_ref_event(channel);
        self._ref = None;
        self.ref_event = None;
        self.received_response = Arc::new(Mutex::new(None));
        self.is_sent = false;
        self.send(client, topic, _ref, current_event, receiver)
            .await
    }

    pub(crate) async fn send(
        &mut self,
        client: &RealtimeClient,
        topic: &str,
        _ref: &str,
        current_event: &str,
        receiver: Receiver<PayloadResponse>,
    ) -> Result<()> {
        if self.has_received(PushReplyStatus::TimedOut).await {
            return Ok(());
        }

        self.start_timeout(_ref, current_event, receiver);
        self.is_sent = true;

        let message = Message {
            topic: String::from(topic),
            payload: self.payload.clone(),
            ref_field: self._ref.clone(),
        };

        client.send(message)
    }

    fn update_payload(&mut self, payload: Payload) {
        // TODO: update without overwriting
        self.payload = payload;
    }

    pub(crate) async fn register_receive_callback(
        &mut self,
        status: PushReplyStatus,
        callback: PushCallback,
    ) {
        let lock_result = self.received_response.lock().await;

        if let Some(payload_response) = &*lock_result {
            callback(payload_response.get_payload());
        }

        let mut rec_hooks_mutex = self.rec_hooks.lock().await;
        rec_hooks_mutex.push(Hook::new(status, callback));
    }

    async fn on_reply(
        payload_response: PayloadResponse,
        received_response: Arc<Mutex<Option<PayloadResponse>>>,
        rec_hooks: Arc<Mutex<Vec<Hook>>>,
    ) {
        // self.cancel_ref_event(channel); // shouldn't need because receiver is dropped when timeout finishes

        *received_response.lock().await = Some(payload_response.clone());

        let rec_hooks_mutex = rec_hooks.lock().await;
        for event in rec_hooks_mutex.iter() {
            if payload_response.get_status() == event.get_status() {
                event.invoke(payload_response.get_payload());
            }
        }
    }

    pub(crate) fn start_timeout(
        &mut self,
        _ref: &str,
        current_event: &str,
        receiver: Receiver<PayloadResponse>,
    ) {
        if self.timeout_abort_handle.is_some() {
            return;
        }

        self._ref = Some(String::from(_ref));
        self.ref_event = Some(String::from(current_event));

        let timeout = self.timeout;
        let event = self.event.clone();
        let received_response = self.received_response.clone();
        let rec_hooks = self.rec_hooks.clone();

        let timeout_handle = tokio::spawn(async move {
            let timeout_result = tokio::time::timeout(timeout, receiver).await;
            match timeout_result {
                Ok(receive_result) => match receive_result {
                    Ok(payload_response) => {
                        Self::on_reply(payload_response, received_response, rec_hooks).await;
                    }
                    Err(error) => {
                        eprintln!("Receiver error in push for event: {event}. Error: {error}.");
                    }
                },
                Err(elapsed) => {
                    println!("Timeout occurred after {elapsed} in push for event: {event}");
                    // TODO: trigger timeout event on channel
                }
            }
        });

        self.timeout_abort_handle = Some(timeout_handle.abort_handle());
    }

    pub(crate) async fn clear_event_callbacks(&mut self) {
        self.rec_hooks.lock().await.clear();
    }

    async fn has_received(&self, status: PushReplyStatus) -> bool {
        let guard = self.received_response.lock().await;
        if let Some(response) = &*guard {
            *response.get_status() == status
        } else {
            false
        }
    }

    pub(crate) fn get_ref(&self) -> &Option<String> {
        &self._ref
    }
}
