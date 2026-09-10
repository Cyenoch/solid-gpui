//! Production host for bounded native commit measurements.
fn main() {
    let monitor = match std::env::var("SOLID_GPUI_PROFILE_HUD").as_deref() {
        Ok("1") => true,
        Ok("0") | Err(std::env::VarError::NotPresent) => false,
        _ => panic!("SOLID_GPUI_PROFILE_HUD must be 0 or 1"),
    };
    solid_gpui::host::run_with_profile(
        solid_gpui::components::host::ComponentHost::new(Vec::new())
            .with_performance_monitor(monitor),
    );
}
