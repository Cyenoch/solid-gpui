use futures::channel::mpsc;
use futures::{SinkExt, Stream, future::poll_fn};
use gpui::{App, Entity};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;

use crate::{ProtocolError, RuntimeAdapter, RuntimeStatus, fatal_runtime_failure};

use super::NativeStateRegistry;

const MAX_MESSAGES_PER_TURN: usize = 16;
const MAX_BYTES_PER_TURN: usize = 1024 * 1024;

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
    messages_in_turn: usize,
    bytes_in_turn: usize,
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
        Ok(Self {
            receiver,
            messages_in_turn: 0,
            bytes_in_turn: 0,
        })
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Message>> {
        // A ready channel never suspends `await`. Bound one foreground poll so
        // a sustained producer cannot starve native input, layout, or painting.
        // A large legal payload is still applied atomically before yielding.
        if self.messages_in_turn >= MAX_MESSAGES_PER_TURN
            || self.bytes_in_turn >= MAX_BYTES_PER_TURN
        {
            self.messages_in_turn = 0;
            self.bytes_in_turn = 0;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        let message = Pin::new(&mut self.receiver).poll_next(cx);
        match &message {
            Poll::Ready(Some(message)) => {
                self.messages_in_turn += 1;
                if let Message::Payload(payload) = message {
                    self.bytes_in_turn += payload.len();
                }
            }
            Poll::Pending | Poll::Ready(None) => {
                self.messages_in_turn = 0;
                self.bytes_in_turn = 0;
            }
        }
        message
    }

    pub(crate) fn attach(
        mut self,
        registry: Entity<NativeStateRegistry>,
        runtime: Arc<dyn RuntimeAdapter>,
        cx: &App,
    ) {
        cx.spawn(async move |cx| {
            while let Some(message) = poll_fn(|cx| self.poll_next(cx)).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::task::{ArcWake, waker_ref};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Default)]
    struct WakeCount(AtomicUsize);

    impl ArcWake for WakeCount {
        fn wake_by_ref(arc_self: &Arc<Self>) {
            arc_self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn ready_commits_yield_with_bounded_work_and_preserve_terminal_order() {
        for payload_sizes in [vec![1; MAX_MESSAGES_PER_TURN], vec![MAX_BYTES_PER_TURN + 1]] {
            let (mut sender, receiver) = mpsc::channel(32);
            for (index, size) in payload_sizes.iter().enumerate() {
                sender
                    .try_send(Message::Payload(vec![index as u8; *size]))
                    .unwrap();
            }
            sender
                .try_send(Message::Terminated(RuntimeStatus::Shutdown))
                .unwrap();
            drop(sender);
            let mut pump = CommitPump {
                receiver,
                messages_in_turn: 0,
                bytes_in_turn: 0,
            };
            let wakes = Arc::new(WakeCount::default());
            let waker = waker_ref(&wakes);
            let mut cx = Context::from_waker(&waker);
            for (index, size) in payload_sizes.iter().enumerate() {
                let Poll::Ready(Some(Message::Payload(payload))) = pump.poll_next(&mut cx) else {
                    panic!("accepted commits must arrive before runtime termination");
                };
                assert_eq!(payload, vec![index as u8; *size]);
            }
            let before_yield = wakes.0.load(Ordering::Relaxed);
            assert!(pump.poll_next(&mut cx).is_pending());
            assert_eq!(wakes.0.load(Ordering::Relaxed), before_yield + 1);
            assert!(matches!(
                pump.poll_next(&mut cx),
                Poll::Ready(Some(Message::Terminated(RuntimeStatus::Shutdown)))
            ));
            assert!(matches!(pump.poll_next(&mut cx), Poll::Ready(None)));
        }
    }
}
