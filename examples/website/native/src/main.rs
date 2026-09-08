fn main() {
    solid_gpui::host::run_with_profile(
        solid_gpui::components::host::ComponentHost::new(vec![
            solid_gpui::components::native_module(),
            website_host::native_module(),
        ])
        .with_performance_monitor(true),
    );
}
