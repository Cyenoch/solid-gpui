fn main() {
    let profile = solid_gpui::components::host::ComponentHost::new(vec![
        solid_gpui::components::native_module(),
        website_host::native_module(),
    ]);
    if let Err(error) = solid_gpui::host::acceptance::run(profile) {
        eprintln!("website-acceptance: {error}");
        std::process::exit(1);
    }
}
