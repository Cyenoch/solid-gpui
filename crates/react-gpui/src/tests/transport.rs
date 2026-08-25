use super::support::*;

#[test]
fn in_memory_runtime_round_trips_framed_commit_and_event() {
    let runtime = InMemoryAdapter::new();
    let snapshot = synthetic_root(1);
    runtime.push_commit(snapshot.encode().unwrap()).unwrap();
    let payload = runtime.recv_commit().unwrap().unwrap();
    assert_eq!(Snapshot::decode(&payload).unwrap(), snapshot);

    let event = Event::press(7, 3, 1, 1, 1, 10);
    runtime.send_event(&event).unwrap();
    assert_eq!(runtime.take_event().unwrap(), Some(event));
    runtime.close().unwrap();
    assert_eq!(runtime.recv_commit().unwrap(), None);
}

#[test]
fn process_runtime_reports_unexpected_clean_eof_and_exit_status() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "exit 37"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();

    assert_eq!(runtime.recv_commit().unwrap(), None);
    let status = runtime.status();
    assert_eq!(
        status,
        RuntimeStatus::Exited {
            code: Some(37),
            signal: None,
        }
    );
    assert!(status.is_failure());
}

#[test]
fn process_runtime_marks_explicit_shutdown_without_failure() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "sleep 10"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();

    runtime.shutdown().unwrap();

    let status = runtime.status();
    assert_eq!(status, RuntimeStatus::Shutdown);
    assert!(!status.is_failure());
}

#[test]
fn process_event_writer_backpressures_bytes_without_blocking_and_shutdown_joins() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "sleep 5"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();
    let event = Event::text_input(
        EVENT_CHANGE,
        7,
        3,
        1,
        1,
        2,
        3,
        TextInputEvent {
            text: "x".repeat(1024 * 1024),
            selection_start: 0,
            selection_end: 0,
            marked_start: None,
            marked_end: None,
            edit_seq: 0,
            reversed: false,
        },
    );

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut accepted = 0;
    let error = loop {
        assert!(
            Instant::now() < deadline,
            "event queue did not backpressure"
        );
        match runtime.send_event(&event) {
            Ok(()) => accepted += 1,
            Err(error) => break error,
        }
    };
    assert!(accepted > 0);
    assert!(error.to_string().contains("byte capacity"));

    let shutdown_started = Instant::now();
    runtime.shutdown().unwrap();
    assert!(shutdown_started.elapsed() < Duration::from_secs(1));
}

#[test]
fn concurrent_process_shutdowns_do_not_deadlock_writer_join() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "sleep 5"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();
    let event = Event::text_input(
        EVENT_CHANGE,
        7,
        3,
        1,
        1,
        2,
        3,
        TextInputEvent {
            text: "x".repeat(1024 * 1024),
            selection_start: 0,
            selection_end: 0,
            marked_start: None,
            marked_end: None,
            edit_seq: 0,
            reversed: false,
        },
    );
    let _ = runtime.send_event(&event);

    let (sender, receiver) = std::sync::mpsc::channel();
    for _ in 0..2 {
        let runtime = Arc::clone(&runtime);
        let sender = sender.clone();
        std::thread::spawn(move || sender.send(runtime.shutdown()).unwrap());
    }
    drop(sender);
    for _ in 0..2 {
        let result = receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("concurrent shutdown timed out");
        assert!(result.is_ok());
    }
}

#[test]
fn process_event_writer_kills_closed_stdin_child_for_reader_eof() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "exec 0<&-; printf '\\000\\000\\000\\000'; sleep 5"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();
    assert_eq!(runtime.recv_commit().unwrap(), Some(Vec::new()));
    let event = Event::press(7, 3, 1, 1, 2, 3);
    let _ = runtime.send_event(&event);

    let (sender, receiver) = std::sync::mpsc::channel();
    let reader_runtime = Arc::clone(&runtime);
    let reader = std::thread::spawn(move || {
        sender
            .send(reader_runtime.recv_commit())
            .expect("send reader result");
    });
    let result = match receiver.recv_timeout(Duration::from_secs(1)) {
        Ok(result) => result,
        Err(error) => {
            let _ = runtime.shutdown();
            let _ = reader.join();
            panic!("writer failure did not close commit reader: {error}");
        }
    };
    reader.join().unwrap();

    assert_eq!(result.unwrap(), None);
    assert!(runtime.event_writer_error().is_some());
    assert_eq!(runtime.status(), RuntimeStatus::Failed);
    let later_error = runtime.send_event(&event).unwrap_err().to_string();
    assert!(later_error.contains("event writer failed"));
    runtime.shutdown().unwrap();
}

#[test]
fn outbound_event_failure_during_shutdown_is_not_fatal() {
    let runtime = InMemoryAdapter::new();
    runtime.close().unwrap();
    let event = Event::press(7, 3, 1, 1, 1, 10);

    assert!(!send_event_or_exit(runtime.as_ref(), "press event", &event));
}

#[test]
fn outbound_event_metadata_includes_wire_identity_without_payload() {
    let event = Event::text_input(
        EVENT_CHANGE,
        7,
        3,
        11,
        19,
        23,
        29,
        TextInputEvent {
            text: "secret input".to_owned(),
            selection_start: 2,
            selection_end: 4,
            marked_start: Some(1),
            marked_end: Some(3),
            edit_seq: 5,
            reversed: false,
        },
    );

    assert_eq!(
        crate::transport::format_event_metadata(&event),
        "event_type=2, surface_id=7, epoch=3, revision=11, sequence=19, node_id=23, listener_id=29"
    );
    assert!(!crate::transport::format_event_metadata(&event).contains("secret input"));
}
