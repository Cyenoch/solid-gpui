use solid_gpui::{ProcessAdapter, gpui::*, native_module};
use std::{
    process::Command,
    sync::atomic::{AtomicU32, Ordering},
};

static CALLS: AtomicU32 = AtomicU32::new(0);

#[native_module(name = "migration")]
mod services {
    use super::*;
    #[command]
    pub fn service_count() -> u32 {
        CALLS.fetch_add(1, Ordering::Relaxed) + 1
    }
}

fn main() {
    solid_gpui::icons::register_icons(&[solid_gpui::icons::IconResource {
        name: "migration:brand",
        svg: include_bytes!("../../assets/brand.svg"),
        color_mode: solid_gpui::icons::IconColorMode::Monochrome,
    }])
    .expect("embedded icon catalog");

    let profile = solid_gpui::components::host::ComponentHost::new(vec![
        solid_gpui::components::native_module(),
        services::native_module(),
    ])
    .with_window_options(|_, cx| {
        let mut options = gpui_component::TitleBar::window_options();
        options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1100.), px(720.)),
            cx,
        )));
        options.window_min_size = Some(size(px(960.), px(640.)));
        options.titlebar.as_mut().unwrap().traffic_light_position = Some(point(px(16.), px(17.)));
        options
    });
    if std::env::args().any(|arg| arg == "--export-native") {
        use solid_gpui::host::HostProfile;
        print!("{}", profile.native_bindings().expect("native contract"));
        return;
    }
    let runtime = if std::env::args().any(|arg| arg == "--production") {
        let mut command = Command::new("bun");
        command.arg("--conditions=browser").arg("dist/main.js");
        ProcessAdapter::spawn(command).expect("Bun runtime")
    } else {
        solid_gpui::runtime::vite::Vite::new(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
            .spawn()
            .expect("Vite runtime")
    };
    solid_gpui::run_application_with_profile(profile, runtime);
}
