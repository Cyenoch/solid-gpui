#[allow(dead_code)]
#[path = "../src/main.rs"]
mod host;
#[test]
fn first_batch_commands_round_trip_through_headless_surface_registry() {
    let mut cx = gpui::TestAppContext::single();
    host::test_support::command_roundtrip(&mut cx);
}
