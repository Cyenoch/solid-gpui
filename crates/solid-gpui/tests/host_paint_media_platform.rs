//! Actual native raster acceptance, distinct from TestAppContext scene assertions.
use gpui::*;
use solid_gpui::native::{decode_json, encode_native_request};
use solid_gpui::protocol::{
    Command, CommandMeta, CommandOperation, DecodedMessage, ExtensionField, ExtensionProperties,
    ExtensionValue,
};
use solid_gpui::{
    HostProperties, InMemoryAdapter, Node, Snapshot, Style, components::host::ComponentHost,
    host::HostProfile,
};
use std::{sync::Arc, time::Duration};

fn extension(
    module: &solid_gpui::native::ModuleDefinition,
    name: &str,
    node_id: u32,
    index: u32,
    props: &[u8],
) -> Node {
    let props: serde_json::Value = decode_json(props).unwrap();
    let mut node = Node::new(node_id, 1, index, solid_gpui::KIND_EXTENSION);
    node.host_properties = Some(HostProperties::Extension(ExtensionProperties {
        provider_id: module.id(),
        catalog_digest: module.digest(),
        entry_id: module.component_id(name).unwrap(),
        entry_version: 1,
        fields: vec![ExtensionField {
            id: 1,
            value: ExtensionValue::Bytes(
                encode_native_request(module.build_digest(), &props).unwrap(),
            ),
        }],
        event_ids: Arc::from([]),
    }));
    node
}
fn main() {
    Application::with_platform(gpui_platform::current_platform(false)).run(|cx| {
        let module = solid_gpui::components::native_module();
        let id = module.id();
        let digest = module.digest();
        let build_digest = module.build_digest();
        let mut parent = Node::new(1, 0, 0, solid_gpui::KIND_VIEW);
        parent.style = Some(Style { width: Some(320.), height: Some(240.),
            background_rgba: Some(0x000000ff), ..Default::default() });
        let paint = extension(&module, "RecordedPaint", 2, 0, br##"{
            "viewportHeight":120,"recording":{"width":320,"height":120,"fit":"none","commands":[
              {"kind":"quad","bounds":{"x":10,"y":10,"width":30,"height":30},"color":{"kind":"solid","color":"#ff0000"}},
              {"kind":"path","points":[{"x":60,"y":10},{"x":100,"y":10},{"x":100,"y":40},{"x":60,"y":40}],"closed":true,"stroke":null,"strokeWidth":0,"fill":{"kind":"solid","color":"#00ff00"}},
              {"kind":"text","origin":{"x":10,"y":65},"text":"Native diagram","fontSize":18,"color":{"kind":"solid","color":"#ffffff"}}
            ]}}
        "##);
        let frame = extension(&module, "LiveFrame", 3, 1, br#"{"viewportHeight":120,"fit":"fill"}"#);
        let mut host = ComponentHost::new(vec![module]);
        host.initialize(cx);
        let (handle, root) = host.open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(point(px(100.), px(100.)), size(px(320.), px(240.))))),
            ..Default::default()
        }, InMemoryAdapter::new(), host.extension_registry(), cx).unwrap();
        handle.update(cx, |_, window, cx| root.update(cx, |root, cx| {
            root.apply_decoded_message_in_window(DecodedMessage::Snapshot(Snapshot::new(1, 1, 0, 1,
                vec![parent, paint, frame])), window, cx).unwrap();
            root.apply_decoded_message_in_window(DecodedMessage::Command(Command::new(
                CommandMeta { surface_id: 1, epoch: 1, after_revision: 1, request_id: 1, node_id: 3 },
                CommandOperation::InvokeNative { module_id: id, module_digest: digest, function_id: 7,
                    args: encode_native_request(build_digest, &serde_json::json!({"sequence":1,"width":1,"height":1,"rgba":[0,0,255,255]})).unwrap() })), window, cx).unwrap();
        })).unwrap();
        cx.spawn(async move |cx| {
            cx.background_executor().timer(Duration::from_millis(250)).await;
            handle.update(cx, |_, window, cx| {
                window.draw(cx).clear(cx);
                let image = window.render_to_image().expect("native platform supports raster capture");
                let scale = window.scale_factor();
                let pixel = |x: f32, y: f32| image.get_pixel((x * scale) as u32, (y * scale) as u32).0;
                let matches = |actual: [u8; 4], expected: [u8; 3]| {
                    for i in 0..3 { assert!((actual[i] as i16 - expected[i] as i16).abs() < 8, "pixel {actual:?}, expected {expected:?}"); }
                };
                matches(pixel(20., 20.), [255, 0, 0]);
                matches(pixel(70., 20.), [0, 255, 0]);
                matches(pixel(160., 180.), [0, 0, 255]);
                let mut text_pixels = 0;
                for y in 65..100 { for x in 10..180 {
                    let p = pixel(x as f32, y as f32);
                    if p[0] > 150 && p[1] > 150 && p[2] > 150 { text_pixels += 1; }
                }}
                assert!(text_pixels > 50, "native text must rasterize visible glyphs");
                if let Some(path) = std::env::var_os("SOLID_GPUI_PAINT_MEDIA_CAPTURE") {
                    image.save(path).unwrap();
                }
            }).unwrap();
            handle.update(cx, |_, window, cx| root.update(cx, |root, cx| {
                for (request_id, function_id) in [(2, 3), (3, 4)] {
                    root.apply_decoded_message_in_window(DecodedMessage::Command(Command::new(
                        CommandMeta { surface_id: 1, epoch: 1, after_revision: 1, request_id, node_id: 3 },
                        CommandOperation::InvokeNative { module_id: id, module_digest: digest, function_id, args: encode_native_request(build_digest, &()).unwrap() })), window, cx).unwrap();
                }
            })).unwrap();
            handle.update(cx, |_, window, cx| {
                window.draw(cx).clear(cx);
                let image = window.render_to_image().unwrap();
                let p = image.get_pixel((160. * window.scale_factor()) as u32, (180. * window.scale_factor()) as u32).0;
                assert!(p[0] < 8 && p[1] < 8 && p[2] < 8, "cleared frame must stop painting blue: {p:?}");
            }).unwrap();
            eprintln!("Native paint/media pixels passed: quad, convex path, shaped text, CPU frame, clear/dispose");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
