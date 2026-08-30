#[allow(dead_code)]
#[path = "../src/main.rs"]
mod host;
#[test]
fn first_batch_commands_round_trip_through_headless_surface_registry() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::command_roundtrip(&mut cx);
}

#[test]
fn virtual_list_scroll_offset_round_trips_wheel_and_precise_command() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::virtual_list_scroll_offset_roundtrip(&mut cx);
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
fn text_file_commands_round_trip_real_utf8_files_asynchronously() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::text_file_command_roundtrip(&mut cx);
}

#[test]
fn load_font_registers_family_and_rasterizes_glyph() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::font_command_roundtrip(&mut cx);
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
#[test]
fn focus_traversal_follows_tree_order_and_wraps() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::focus_traversal_roundtrip(&mut cx);
}

#[test]
fn disabled_pressable_is_skipped_by_focus_traversal() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::disabled_pressable_is_skipped_roundtrip(&mut cx);
}

#[test]
fn conditionally_mounted_focus_node_keeps_tree_order() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::conditional_focus_mount_keeps_tree_order(&mut cx);
}

#[test]
fn focused_unmount_blurs_and_restores_focus() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::focused_unmount_blurs_and_restores_ancestor(&mut cx);
}
#[test]
fn pressable_focusable_updates_are_accepted_and_focusable() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::pressable_focusable_update_roundtrip(&mut cx);
}

#[test]
fn close_policy_simulate_close_requires_and_resolves_confirmation() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::close_policy_simulate_close_roundtrip(&mut cx);
}

#[test]
fn closing_one_surface_keeps_host_alive_and_last_surface_requests_quit() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::last_surface_close_requests_application_quit(&mut cx);
}
