//! Production host for bounded native commit measurements, without the FPS overlay.
fn main() {
    solid_gpui::host::run_with_profile(
        solid_gpui::components::host::ComponentHost::new(Vec::new()),
    );
}
