fn main() {
    solid_gpui::host::acceptance::run(solid_gpui::components::host::ComponentHost::new(vec![
        solid_gpui::components::native_module(),
        website_host::native_module(),
    ]));
}
