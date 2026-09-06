use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AppContext, ExternalDragPayload, ExternalPaths, FileDragPaths, IntoElement, ParentElement,
    Pixels, Render, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgba,
};

use crate::protocol::HostProperties;
use crate::tree::StoredNode;

use super::super::SolidRoot;
use super::super::events::{emit_drag_drop, emit_drag_over, emit_external_file_drop};

#[derive(Clone)]
struct SolidDragPayload {
    drag_type: String,
}

struct DragPreview {
    position: gpui::Point<Pixels>,
}

impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .pl(self.position.x - px(48.0))
            .pt(self.position.y - px(16.0))
            .child(
                div()
                    .w(px(96.0))
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(6.0))
                    .border(px(1.0))
                    .border_color(rgba(0x64748bff))
                    .bg(rgba(0x334155ff))
                    .text_color(rgba(0xf8fafcff))
                    .text_size(px(11.0))
                    .line_height(px(16.0))
                    .child(SharedString::new_static("Moving item")),
            )
    }
}

pub(super) fn external_file_drag_payload(paths: &[String]) -> ExternalDragPayload {
    let entries = paths.iter().map(|path| {
        let path = PathBuf::from(path);
        let is_directory = std::fs::metadata(&path)
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false);
        (path, is_directory)
    });
    ExternalDragPayload::Files(FileDragPaths::new(entries))
}

pub(super) fn apply<E: StatefulInteractiveElement>(
    mut element: E,
    root: &SolidRoot,
    node: &StoredNode,
) -> E {
    let Some(HostProperties::Drag(drag)) = node.host_properties.as_ref() else {
        return element;
    };
    if let Some(drag_type) = drag.drag_type.clone() {
        let active_drag_type = Rc::clone(&root.active_drag_type);
        element = element.on_drag(
            SolidDragPayload { drag_type },
            move |value, position, _, cx| {
                *active_drag_type.borrow_mut() = Some(value.drag_type.clone());
                cx.new(move |_| DragPreview { position })
            },
        );
    }
    if let Some(export_files) = drag.export_files.clone() {
        element = element.external_drag_payload(move |_payload: &SolidDragPayload, _, _| {
            Some(external_file_drag_payload(&export_files))
        });
    }
    // Drag host properties also cover external-file-only targets.
    // Capability bits keep internal notifications on the callbacks
    // that requested them.
    if node.listener_id != 0 && drag.accepts_drag_over {
        let active_drag_type = Rc::clone(&root.active_drag_type);
        let runtime = Arc::clone(&root.runtime);
        let sequence = Arc::clone(&root.next_sequence);
        let surface_id = root.store.surface_id();
        let epoch = root.store.epoch();
        let revision = root.store.revision();
        let node_id = node.id;
        let listener_id = node.listener_id;
        element = element.on_mouse_move(move |_event, _, cx| {
            if !cx.has_active_drag() {
                active_drag_type.borrow_mut().take();
                return;
            }
            let Some(drag_type) = active_drag_type.borrow().clone() else {
                return;
            };
            emit_drag_over(
                runtime.as_ref(),
                sequence.as_ref(),
                surface_id,
                epoch,
                revision,
                node_id,
                listener_id,
                &drag_type,
            );
        });
    }
    if node.listener_id != 0 && drag.accepts_drop {
        let active_drag_type = Rc::clone(&root.active_drag_type);
        let runtime = Arc::clone(&root.runtime);
        let sequence = Arc::clone(&root.next_sequence);
        let surface_id = root.store.surface_id();
        let epoch = root.store.epoch();
        let revision = root.store.revision();
        let node_id = node.id;
        let listener_id = node.listener_id;
        element = element.on_drop(move |drag: &SolidDragPayload, _, _| {
            active_drag_type.borrow_mut().take();
            emit_drag_drop(
                runtime.as_ref(),
                sequence.as_ref(),
                surface_id,
                epoch,
                revision,
                node_id,
                listener_id,
                &drag.drag_type,
            );
        });
    }
    if node.listener_id != 0 {
        let active_drag_type = Rc::clone(&root.active_drag_type);
        let runtime = Arc::clone(&root.runtime);
        let sequence = Arc::clone(&root.next_sequence);
        let surface_id = root.store.surface_id();
        let epoch = root.store.epoch();
        let revision = root.store.revision();
        let node_id = node.id;
        let listener_id = node.listener_id;
        element = element.on_drop(move |paths: &ExternalPaths, _, _| {
            active_drag_type.borrow_mut().take();
            emit_external_file_drop(
                runtime.as_ref(),
                sequence.as_ref(),
                surface_id,
                epoch,
                revision,
                node_id,
                listener_id,
                paths
                    .paths()
                    .iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect(),
            );
        });
    }
    element
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outbound_file_drag_payload_preserves_paths_and_directory_metadata() {
        let missing = "/definitely/not/a/solid-gpui-file";
        let payload = external_file_drag_payload(&["/tmp".to_owned(), missing.to_owned()]);
        let ExternalDragPayload::Files(paths) = payload;
        assert_eq!(
            paths.entries(),
            &[
                (std::path::PathBuf::from("/tmp"), true),
                (std::path::PathBuf::from(missing), false),
            ]
        );
    }
}
