#[allow(dead_code)]
#[path = "../src/main.rs"]
mod host;
#[test]
fn first_batch_commands_round_trip_through_headless_surface_registry() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::command_roundtrip(&mut cx);
}

#[test]
fn dialog_commands_round_trip_paths_cancellation_and_close_safety() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::dialog_command_roundtrip(&mut cx);
}

#[test]
fn notification_action_and_body_responses_round_trip_to_the_origin_surface() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::notification_response_roundtrip(&mut cx);
}
