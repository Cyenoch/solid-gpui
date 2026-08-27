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
#[test]
fn keybinding_commands_register_replace_and_route_actions_headlessly() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::keybinding_roundtrip(&mut cx);
}

#[test]
fn renderer_termination_closes_surfaces_without_reentrant_registry_update() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::renderer_termination_closes_surfaces_without_reentrant_update(&mut cx);
}
#[test]
fn focus_and_blur_events_remain_routed_to_each_surface() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::cross_surface_focus_blur_roundtrip(&mut cx);
}
