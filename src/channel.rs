use std::{
    collections::HashMap,
    mem::{Discriminant, discriminant},
    sync::Arc,
};

use tokio::sync::{
    Mutex,
    oneshot::{Receiver, Sender, channel},
};

use crate::{
    channel_event::ChannelEvent,
    client::RealtimeClient,
    error::RealtimeError,
    protocol_objects::{
        Broadcast, JoinConfig, Payload, PhxClose, PhxError, PhxJoin, PhxLeave, PhxReply,
        PhxResponse,
    },
    push::Push,
    types::{
        Binding, ChannelState, PayloadResponse, PushReplyStatus, Result, SubscribeCallback,
        SubscribeState,
    },
    utils::get_reply_event_name,
};

#[derive(Clone)]
pub struct RealtimeChannel {
    topic: String,
    config: JoinConfig,
    join_push: Push,
    mutable_state: Arc<Mutex<RealtimeChannelMutableState>>,
}

#[derive(Default)]
pub(crate) struct RealtimeChannelMutableState {
    state: ChannelState,
    joined_once: bool,
    push_buffer: Vec<Push>,
    bindings: HashMap<Discriminant<Payload>, Vec<Binding>>,
    push_senders: HashMap<String, Sender<PayloadResponse>>,
}

impl RealtimeChannel {
    pub(crate) fn new(client: &RealtimeClient, topic: &str, config: Option<JoinConfig>) -> Self {
        let join_push = Push::new(
            "phx_join",
            Payload::PhxJoin(PhxJoin {
                config: config.clone().unwrap_or_default(),
                access_token: String::from(client.get_access_token()),
            }),
            None,
        );

        Self {
            topic: String::from(topic),
            config: config.unwrap_or_default(),
            join_push,
            mutable_state: Arc::new(Mutex::new(RealtimeChannelMutableState::default())),
        }
    }

    pub(crate) async fn register_default_events(&self) {
        self.register_event(
            discriminant(&Payload::PhxClose(PhxClose)),
            ChannelEvent::Close(Box::new(Self::on_close)),
        )
        .await;
        self.register_event(
            discriminant(&Payload::PhxError(PhxError)),
            ChannelEvent::Error(Box::new(Self::on_error)),
        )
        .await;
        self.register_event(
            discriminant(&Payload::PhxReply(PhxReply::Ok(PhxResponse::default()))),
            ChannelEvent::Reply(Box::new(Self::on_reply)),
        )
        .await;
    }

    fn on_close(state: &mut RealtimeChannelMutableState, should_remove_channel: &mut bool) {
        // reset the rejoin timer
        state.state = ChannelState::Closed;

        *should_remove_channel = true;
    }

    fn on_error(state: &mut RealtimeChannelMutableState) {
        if state.state == ChannelState::Closed || state.state == ChannelState::Leaving {
            return;
        }

        state.state = ChannelState::Errored;
        // rejoin
    }

    fn on_reply(state: &mut RealtimeChannelMutableState, payload: Payload, _ref: Option<&str>) {
        if let Payload::PhxReply(ref reply) = payload {
            match reply {
                PhxReply::Ok(_) => {
                    if let Some(_ref) = _ref {
                        let event_name = get_reply_event_name(_ref);
                        if let Some(sender) = state.push_senders.remove(&event_name)
                            && let Err(failed_to_send_payload) =
                                sender.send(PayloadResponse::new(PushReplyStatus::Ok, payload))
                        {
                            // TODO: error handling
                            println!("Reply received for a push that was already dropped.");
                        }
                    }
                }
                PhxReply::Error(_) => {
                    if let Some(_ref) = _ref {
                        let event_name = get_reply_event_name(_ref);
                        if let Some(sender) = state.push_senders.remove(&event_name)
                            && let Err(failed_to_send_payload) =
                                sender.send(PayloadResponse::new(PushReplyStatus::Error, payload))
                        {
                            // TODO: error handling
                            println!("Reply received for a push that was already dropped.");
                        }
                    }
                }
            }
        }
    }

    pub async fn subscribe(
        &mut self,
        client: &mut RealtimeClient,
        callback: Option<SubscribeCallback>,
    ) -> Result<()> {
        if !client.is_connected() {
            client.connect().await?;
        }

        if self.mutable_state.lock().await.joined_once {
            return Err(RealtimeError::MultipleSubscriptionError);
        }

        // let mut config_payload = self.config.clone();

        // let postgres_changes: Vec<_> = self
        //     .events
        //     .iter()
        //     .filter_map(|(event_type, _)| {
        //         if event_type == "postgres_changes" {
        //             // Extract filter from the event binding
        //             // This would need to be implemented based on your event structure
        //             None // Placeholder for now
        //         } else {
        //             None
        //         }
        //     })
        //     .collect();

        // update payload with the config
        self.mutable_state.lock().await.joined_once = true;

        let callback = Arc::new(callback);
        let on_join_push_ok = {
            let callback = Arc::clone(&callback);
            move |_: &Payload| {
                if let Some(ref callback) = *callback {
                    callback(Ok(SubscribeState::Subscribed));
                }
            }
        };

        let on_join_push_error = {
            let callback = Arc::clone(&callback);
            move |payload: &Payload| {
                println!("Calling 'on_join_push_error' PushEvent.");
                if let Some(ref callback) = *callback {
                    let payload_raw = serde_json::to_string(payload).unwrap_or(String::from(
                        "Subscription error payload failed to deserialize.",
                    ));
                    callback(Err(RealtimeError::SubscribeError {
                        payload: payload_raw,
                    }))
                }
            }
        };

        let on_join_push_timeout = move |_: &Payload| {
            println!("Calling 'on_join_push_timeout' PushEvent.");
            if let Some(ref callback) = *callback {
                callback(Ok(SubscribeState::TimedOut));
            }
        };

        self.join_push
            .register_receive_callback(PushReplyStatus::Ok, Box::new(on_join_push_ok))
            .await;
        self.join_push
            .register_receive_callback(PushReplyStatus::Error, Box::new(on_join_push_error))
            .await;
        self.join_push
            .register_receive_callback(PushReplyStatus::TimedOut, Box::new(on_join_push_timeout))
            .await;

        let (sender, receiver) = channel();
        let _ref = client.make_ref();
        let reply_event_name = get_reply_event_name(&_ref);

        {
            let mut mutable_state = self.mutable_state.lock().await;
            mutable_state
                .push_senders
                .insert(reply_event_name.clone(), sender);
        }

        self.rejoin(client, &_ref, receiver).await?;

        Ok(())
    }

    async fn push(
        &self,
        client: &mut RealtimeClient,
        event: &str,
        payload: Payload,
        // timeout
    ) -> Result<()> {
        if !self.mutable_state.lock().await.joined_once {
            return Err(RealtimeError::PushWhileUnsubscribedError {
                event: String::from(event),
                topic: self.topic.clone(),
            });
        }

        let mut push = Push::new(event, payload, None);

        let (sender, receiver) = channel();
        let _ref = client.make_ref();
        let reply_event_name = get_reply_event_name(&_ref);

        {
            let mut mutable_state = self.mutable_state.lock().await;
            mutable_state
                .push_senders
                .insert(reply_event_name.clone(), sender);
        }

        if self.can_push(client).await {
            return push
                .send(client, &self.topic, &_ref, &reply_event_name, receiver)
                .await;
        } else {
            println!("Didn't push event {reply_event_name} because client was not connected.");
            // push.start_timeout(self, client, receiver);
            // self.push_buffer.push(push);
        }

        Ok(())
    }

    pub(crate) async fn register_event(
        &self,
        discriminant: Discriminant<Payload>,
        event: ChannelEvent,
    ) {
        let mut mutable_state = self.mutable_state.lock().await;

        let binding = Binding::new(event, None);
        mutable_state
            .bindings
            .entry(discriminant)
            .or_default()
            .push(binding);
    }

    pub async fn on_broadcast<F>(&self, event: &str, f: F)
    where
        F: Fn(Payload) + Send + Sync + 'static,
    {
        self.register_event(
            discriminant(&Payload::Broadcast(Broadcast::default())),
            ChannelEvent::Broadcast(Box::new(f)),
        )
        .await
    }

    pub async fn send_broadcast(
        &self,
        client: &mut RealtimeClient,
        event: &str,
        payload: Payload,
    ) -> Result<()> {
        self.push(client, event, payload).await
    }

    pub fn get_topic(&self) -> &str {
        &self.topic
    }

    async fn rejoin(
        &mut self,
        client: &mut RealtimeClient,
        _ref: &str,
        receiver: Receiver<PayloadResponse>,
    ) -> Result<()> {
        let mut mutable_state = self.mutable_state.lock().await;
        if mutable_state.state == ChannelState::Leaving {
            return Ok(());
        }

        // TODO: client leave open topic
        mutable_state.state = ChannelState::Joining;
        self.join_push
            .resend(
                client,
                &self.topic,
                _ref,
                &get_reply_event_name(_ref),
                receiver,
            )
            .await
    }

    async fn can_push(&self, client: &RealtimeClient) -> bool {
        client.is_connected() && self.mutable_state.lock().await.joined_once
    }

    pub(crate) async fn trigger(
        &self,
        payload: Payload,
        _ref: Option<&str>,
        should_remove_channel: &mut bool,
    ) {
        let maybe_ignore = [
            discriminant(&Payload::PhxClose(PhxClose)),
            discriminant(&Payload::PhxError(PhxError)),
            discriminant(&Payload::PhxLeave(PhxLeave)),
            discriminant(&Payload::PhxJoin(PhxJoin::default())),
        ];

        if let Some(_ref) = _ref
            && let Some(join_push_ref) = self.join_push.get_ref()
            && maybe_ignore.contains(&discriminant(&payload))
            && _ref != join_push_ref
        {
            println!("Ignoring trigger for {payload} payload with ref {_ref}.");
            return;
        }

        let mut mutable_state = self.mutable_state.lock().await;

        if let Some(bindings) = mutable_state.bindings.remove(&discriminant(&payload)) {
            for binding in &bindings {
                binding.invoke(
                    &mut mutable_state,
                    payload.clone(),
                    _ref,
                    should_remove_channel,
                );
            }
            mutable_state
                .bindings
                .insert(discriminant(&payload), bindings);
        }
    }
}
