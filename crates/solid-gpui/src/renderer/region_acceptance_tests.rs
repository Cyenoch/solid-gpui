use super::*;
use crate::protocol::{DecodedMessage, Patch, PatchOperation};
use gpui::TestAppContext;

#[gpui::test]
fn acceptance_replays_only_painted_region_content_and_bounds(cx: &mut TestAppContext) {
    let window = cx.open_window(gpui::size(px(800.0), px(600.0)), |_, _| {
        let mut root = SolidRoot::new(crate::InMemoryAdapter::new());
        root.enable_acceptance();
        root
    });
    let root = window.root(cx).unwrap();
    root.update(cx, |root, cx| {
        root.apply_decoded_message(DecodedMessage::Snapshot(region_tests::fixture(10)), cx)
    })
    .unwrap();
    region_tests::draw(cx, window.into());
    let painted = root.read_with(cx, |root, _| {
        root.acceptance_nodes()
            .into_iter()
            .find(|node| node.id == 10)
            .unwrap()
    });
    root.update(cx, |root, cx| {
        root.primitive_constructions.set(0);
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                7,
                3,
                1,
                2,
                vec![region_tests::update(4, "Count: 12")],
            )),
            cx,
        )
    })
    .unwrap();
    region_tests::draw(cx, window.into());
    root.read_with(cx, |root, _| {
        let nodes = root.acceptance_nodes();
        let retained = nodes
            .iter()
            .find(|node| node.id == 10)
            .expect("scene replay preserves painted observations");
        assert_eq!(retained.text, painted.text);
        assert_eq!(retained.bounds.x, painted.bounds.x);
        assert_eq!(retained.bounds.width, painted.bounds.width);
        assert_eq!(
            nodes
                .iter()
                .find(|node| node.id == 3)
                .unwrap()
                .text
                .as_deref(),
            Some("Count: 12")
        );
        assert!(root.primitive_constructions.get() <= 4);
    });
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                7,
                3,
                2,
                3,
                vec![region_tests::update(11, "Newly painted region text")],
            )),
            cx,
        )
    })
    .unwrap();
    region_tests::draw(cx, window.into());
    root.read_with(cx, |root, _| {
        assert_eq!(
            root.acceptance_nodes()
                .into_iter()
                .find(|node| node.id == 10)
                .unwrap()
                .text
                .as_deref(),
            Some("Newly painted region text")
        )
    });
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                7,
                3,
                3,
                4,
                vec![PatchOperation::Delete { id: 2 }],
            )),
            cx,
        )
    })
    .unwrap();
    region_tests::draw(cx, window.into());
    root.read_with(cx, |root, _| {
        assert!(!root.acceptance_nodes().iter().any(|node| node.id == 10))
    });
}
