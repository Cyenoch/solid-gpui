use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gpui::{App, Entity};
use std::sync::Arc;
use std::thread;

use crate::{ProtocolError, RuntimeAdapter, RuntimeStatus, fatal_runtime_failure};

use super::NativeStateRegistry;

enum Message {
    Payload(Vec<u8>),
    Terminated(RuntimeStatus),
    Transport(ProtocolError),
}

/// Bridges the blocking RuntimeAdapter commit stream to the GPUI foreground
/// executor. The reader thread only handles bytes; all native state remains on
/// the foreground executor that owns the registry.
pub(crate) struct CommitPump {
    receiver: mpsc::Receiver<Message>,
}

impl CommitPump {
    pub(crate) fn start(runtime: Arc<dyn RuntimeAdapter>) -> Result<Self, String> {
        let (mut sender, receiver) = mpsc::channel::<Message>(32);
        thread::Builder::new()
            .name("solid-gpui-host-commit-pump".to_owned())
            .spawn(move || {
                loop {
                    match runtime.recv_commit() {
                        Ok(Some(payload)) => {
                            runtime.tap_inbound_payload(&payload);
                            if futures::executor::block_on(sender.send(Message::Payload(payload)))
                                .is_err()
                            {
                                break;
                            }
                        }
                        Ok(None) => {
                            let status = match runtime.status() {
                                RuntimeStatus::Running => RuntimeStatus::Exited {
                                    code: None,
                                    signal: None,
                                },
                                status => status,
                            };
                            let _ = futures::executor::block_on(
                                sender.send(Message::Terminated(status)),
                            );
                            break;
                        }
                        Err(error) => {
                            let _ =
                                futures::executor::block_on(sender.send(Message::Transport(error)));
                            break;
                        }
                    }
                }
            })
            .map_err(|error| format!("failed to start Solid GPUI commit pump: {error}"))?;
        Ok(Self { receiver })
    }

    pub(crate) fn attach(
        mut self,
        registry: Entity<NativeStateRegistry>,
        runtime: Arc<dyn RuntimeAdapter>,
        cx: &App,
    ) {
        cx.spawn(async move |cx| {
            while let Some(message) = self.receiver.next().await {
                match message {
                    Message::Payload(payload) => {
                        let result = registry
                            .update(cx, |registry, cx| registry.route_payload(&payload, cx));
                        if let Err(error) = result {
                            fatal_runtime_failure(
                                runtime.as_ref(),
                                "rejected renderer commit",
                                error,
                            );
                        }
                    }
                    Message::Terminated(status) => {
                        registry.update(cx, |registry, cx| registry.close_all(cx));
                        if status.is_failure() {
                            fatal_runtime_failure(
                                runtime.as_ref(),
                                "renderer runtime terminated unexpectedly",
                                status,
                            );
                        }
                        break;
                    }
                    Message::Transport(error) => {
                        registry.update(cx, |registry, cx| registry.close_all(cx));
                        fatal_runtime_failure(runtime.as_ref(), "renderer transport error", error);
                    }
                }
            }
        })
        .detach();
    }
}
