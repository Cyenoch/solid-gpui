fn main() {
    #[cfg(feature = "gpui-component")]
    solid_gpui::components::host::run();
    #[cfg(not(feature = "gpui-component"))]
    solid_gpui::host::run_default();
}
