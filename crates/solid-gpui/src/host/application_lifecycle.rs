//! Application lifetime and acknowledged activation delivery, independent of windows.
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub(super) struct Activation {
    pub sequence: u32,
    pub reason: &'static str,
    pub urls: Vec<String>,
    sent_epoch: u32,
}

#[derive(Clone)]
pub(super) struct ApplicationLifecycle {
    pub epoch: u32,
    pub keep_alive: bool,
    next_sequence: u32,
    acknowledged: u32,
    sent_sequence: u32,
    pending: VecDeque<Activation>,
}

impl Default for ApplicationLifecycle {
    fn default() -> Self {
        let mut state = Self {
            epoch: 0,
            keep_alive: false,
            next_sequence: 1,
            acknowledged: 0,
            sent_sequence: 0,
            pending: VecDeque::new(),
        };
        state
            .enqueue("launch", Vec::new())
            .expect("valid launch activation");
        state
    }
}

impl ApplicationLifecycle {
    pub fn enqueue(&mut self, reason: &'static str, urls: Vec<String>) -> Result<(), String> {
        if !matches!(reason, "launch" | "reopen" | "open-urls")
            || (reason == "open-urls") == urls.is_empty()
            || urls.len() > 64
            || urls
                .iter()
                .any(|url| url.is_empty() || url.len() > 4096 || url.chars().any(char::is_control))
        {
            return Err("activation requires at most 64 safe URLs of at most 4096 bytes".into());
        }
        if self.pending.len() >= 32 {
            return Err("application activation queue is full".into());
        }
        let next = self
            .next_sequence
            .checked_add(1)
            .ok_or("application activation sequence exhausted")?;
        self.pending.push_back(Activation {
            sequence: self.next_sequence,
            reason,
            urls,
            sent_epoch: 0,
        });
        self.next_sequence = next;
        Ok(())
    }

    pub fn configure(
        &mut self,
        epoch: u32,
        keep_alive: bool,
        quit: bool,
        acknowledged: u32,
    ) -> Result<bool, String> {
        if epoch < self.epoch {
            return Ok(false);
        }
        if epoch == 0 || acknowledged > self.sent_sequence || acknowledged < self.acknowledged {
            return Err("invalid application generation or activation acknowledgement".into());
        }
        self.epoch = epoch;
        self.keep_alive = keep_alive;
        self.acknowledged = acknowledged;
        self.pending
            .retain(|activation| activation.sequence > acknowledged);
        Ok(quit)
    }

    pub fn pending(&self) -> Vec<Activation> {
        if self.epoch == 0 {
            return Vec::new();
        }
        self.pending
            .iter()
            .filter(|activation| activation.sent_epoch != self.epoch)
            .cloned()
            .collect()
    }

    pub fn sent(&mut self, sequence: u32) {
        if let Some(activation) = self
            .pending
            .iter_mut()
            .find(|item| item.sequence == sequence)
        {
            activation.sent_epoch = self.epoch;
            self.sent_sequence = self.sent_sequence.max(sequence);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn activations_wait_for_ready_replay_across_reload_and_retire_only_after_ack() {
        let mut state = ApplicationLifecycle::default();
        state
            .enqueue("open-urls", vec!["demo://document/7".into()])
            .unwrap();
        assert!(state.pending().is_empty());
        assert!(!state.configure(1, true, false, 0).unwrap());
        assert_eq!(state.pending().len(), 2);
        state.sent(1);
        state.sent(2);
        assert!(state.pending().is_empty());
        state.configure(2, true, false, 1).unwrap();
        assert_eq!(state.pending()[0].sequence, 2);
        assert!(!state.configure(1, false, true, 0).unwrap());
        assert!(state.keep_alive);
        state.configure(2, true, false, 2).unwrap();
        assert!(state.pending().is_empty());
        assert!(state.configure(2, false, true, 3).is_err());
        assert!(state.configure(2, false, true, 2).unwrap());
        for _ in 0..32 {
            state.enqueue("reopen", Vec::new()).unwrap();
        }
        assert!(state.enqueue("reopen", Vec::new()).is_err());
    }
}
