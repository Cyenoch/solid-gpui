fn main() {
    if let Err(error) = solid_gpui::host::acceptance::run(solid_gpui::host::DefaultHostProfile) {
        eprintln!("solid-gpui-acceptance: {error}");
        std::process::exit(1);
    }
}
