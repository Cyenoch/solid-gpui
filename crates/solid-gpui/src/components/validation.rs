//! Property bounds are checked while decoding, before publishing a candidate tree.
use crate::native::{Deserialize, Serialize, TS};
macro_rules! bounded {
    ($name:ident, $ty:ty, $min:expr, $max:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
        pub struct $name(pub $ty);
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let value = <$ty>::deserialize(d)?;
                if ($min..=$max).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(serde::de::Error::custom(concat!(
                        stringify!($name),
                        " is outside its supported range"
                    )))
                }
            }
        }
    };
}
bounded!(RatingMax, u32, 0, 100);
bounded!(PageButtons, u32, 5, 64);
bounded!(GridColumns, u16, 1, 64);
bounded!(GridSpan, u16, 1, 64);
bounded!(GridLine, i16, -65, 65);
bounded!(DescriptionColumns, u16, 1, 10);
bounded!(LogicalPixels, f32, 0., 1_000_000.);

#[cfg(test)]
mod tests {
    use crate::{
        DecodedMessage, HostProperties, InMemoryAdapter, KIND_EXTENSION, KIND_VIEW, Node, Snapshot,
        SolidRoot,
        native::ModuleDefinition,
        protocol::{ExtensionField, ExtensionProperties, ExtensionValue},
    };
    use gpui::{AppContext, TestAppContext};
    use std::{rc::Rc, sync::Arc};
    fn node(
        m: &ModuleDefinition,
        id: u32,
        parent: u32,
        index: u32,
        name: &str,
        json: &str,
    ) -> Node {
        let mut node = Node::new(id, parent, index, KIND_EXTENSION);
        node.host_properties = Some(HostProperties::Extension(ExtensionProperties {
            provider_id: m.id(),
            catalog_digest: m.digest(),
            entry_id: m.component_id(name).unwrap(),
            entry_version: 1,
            fields: vec![ExtensionField {
                id: 1,
                value: ExtensionValue::Bytes(json.as_bytes().to_vec()),
            }],
            event_ids: Arc::from([]),
        }));
        node
    }
    #[gpui::test]
    fn amplified_props_and_parent_span_reject_before_publication(cx: &mut TestAppContext) {
        let module = Rc::new(super::super::native_module());
        let root = cx.new(|_| SolidRoot::with_extensions(InMemoryAdapter::new(), module.clone()));
        root.update(cx, |r, cx| {
            r.apply_decoded_message(
                DecodedMessage::Snapshot(Snapshot::new(
                    1,
                    1,
                    0,
                    1,
                    vec![Node::new(1, 0, 0, KIND_VIEW)],
                )),
                cx,
            )
        })
        .unwrap();
        for (name, json, slots) in [
            ("Rating", r#"{"max":4294967295}"#, 0),
            (
                "Pagination",
                r#"{"totalPages":4294967295,"visiblePages":4294967295}"#,
                0,
            ),
            ("Form", r#"{"columns":65535}"#, 0),
            ("Field", r#"{"colSpan":65535}"#, 3),
            ("Field", r#"{"colStart":-32768}"#, 3),
            ("Form", r#"{"labelWidth":-1}"#, 0),
            ("DescriptionList", r#"{"labelWidth":-1}"#, 0),
            ("WindowBorder", r#"{"shadowSize":-1}"#, 0),
            ("WindowBorder", r#"{"resizeHitSize":-1}"#, 0),
        ] {
            let mut nodes = vec![
                Node::new(1, 0, 0, KIND_VIEW),
                node(&module, 2, 1, 0, name, json),
            ];
            nodes.extend((0..slots).map(|i| Node::new(3 + i, 2, i, KIND_VIEW)));
            assert!(
                root.update(cx, |r, cx| r.apply_decoded_message(
                    DecodedMessage::Snapshot(Snapshot::new(1, 2, 0, 1, nodes)),
                    cx
                ))
                .is_err(),
                "{name}: {json}"
            );
            root.read_with(cx, |r, _| {
                assert_eq!(r.store().revision(), 1);
                assert!(r.store().get(2).is_none());
            });
        }
        let description = |columns| {
            vec![
                Node::new(1, 0, 0, KIND_VIEW),
                node(
                    &module,
                    2,
                    1,
                    0,
                    "DescriptionList",
                    &format!(r#"{{"columns":{columns}}}"#),
                ),
                node(&module, 3, 2, 0, "DescriptionItem", r#"{"span":2}"#),
                Node::new(4, 3, 0, KIND_VIEW),
                Node::new(5, 3, 1, KIND_VIEW),
            ]
        };
        assert!(
            root.update(cx, |r, cx| r.apply_decoded_message(
                DecodedMessage::Snapshot(Snapshot::new(1, 2, 0, 1, description(1))),
                cx
            ))
            .is_err()
        );
        root.read_with(cx, |r, _| assert_eq!(r.store().revision(), 1));
        root.update(cx, |r, cx| {
            r.apply_decoded_message(
                DecodedMessage::Snapshot(Snapshot::new(1, 2, 0, 1, description(2))),
                cx,
            )
        })
        .unwrap();
        // A huge total remains legal: ellipses now own a constant-size page input.
        for compact in [false, true] {
            let nodes = vec![
                Node::new(1, 0, 0, KIND_VIEW),
                node(
                    &module,
                    2,
                    1,
                    0,
                    "Pagination",
                    &format!(r#"{{"totalPages":4294967295,"visiblePages":5,"compact":{compact}}}"#),
                ),
            ];
            root.update(cx, |r, cx| {
                r.apply_decoded_message(
                    DecodedMessage::Snapshot(Snapshot::new(1, 3 + u32::from(compact), 0, 1, nodes)),
                    cx,
                )
            })
            .unwrap();
        }
    }
}
