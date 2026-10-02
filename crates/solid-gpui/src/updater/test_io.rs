//! Invocation-thread fault injection; never linked into production hosts.

use std::{cell::RefCell, collections::VecDeque, sync::mpsc};

enum Action {
    Fail(&'static str),
    Pause(&'static str, mpsc::Sender<()>, mpsc::Receiver<()>),
}

thread_local! {
    static ACTIONS: RefCell<VecDeque<Action>> = const { RefCell::new(VecDeque::new()) };
}

pub(super) fn failures(points: &[&'static str]) {
    ACTIONS
        .with(|actions| *actions.borrow_mut() = points.iter().copied().map(Action::Fail).collect());
}

pub(super) fn exhausted() {
    ACTIONS.with(|actions| {
        assert!(
            actions.borrow().is_empty(),
            "fault injection point was not reached"
        )
    });
}

pub(super) fn pause(point: &'static str, started: mpsc::Sender<()>, release: mpsc::Receiver<()>) {
    ACTIONS.with(|actions| {
        actions
            .borrow_mut()
            .push_back(Action::Pause(point, started, release))
    });
}

pub(super) fn checkpoint(point: &str) -> Result<(), String> {
    let action = ACTIONS.with(|actions| {
        let mut actions = actions.borrow_mut();
        let matches = match actions.front() {
            Some(Action::Fail(name) | Action::Pause(name, _, _)) => *name == point,
            None => false,
        };
        matches.then(|| actions.pop_front().unwrap())
    });
    match action {
        Some(Action::Fail(_)) => Err(format!("injected {point} failure")),
        Some(Action::Pause(_, started, release)) => {
            started.send(()).unwrap();
            release
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            Ok(())
        }
        None => Ok(()),
    }
}
