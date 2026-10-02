//! Generated commands for native-owned document selection and literal search.
use super::*;
use gpui::{ClipboardItem, Styled};

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextPosition {
    pub node_id: u32,
    /// UTF-16 offset, required to be a grapheme boundary.
    pub offset: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextSpan {
    pub node_id: u32,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TextSelectionSnapshot {
    pub text_revision: u32,
    pub selection_revision: u32,
    pub anchor: Option<TextPosition>,
    pub head: Option<TextPosition>,
    pub spans: Vec<TextSpan>,
    pub text: String,
    /// Selected virtual rows have left the committed materialization window.
    pub detached: bool,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextSelectionRequest {
    pub text_revision: u32,
    pub anchor: TextPosition,
    pub head: TextPosition,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextSearchRequest {
    pub query: String,
}

#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TextSearchSnapshot {
    pub text_revision: u32,
    pub search_revision: u32,
    pub query: String,
    pub matches: Vec<Vec<TextSpan>>,
    pub active_match: Option<u32>,
    pub truncated: bool,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextSearchSelection {
    pub text_revision: u32,
    pub search_revision: u32,
    pub match_index: u32,
}

#[derive(Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct TextSelectionObserverProps {}

#[derive(Clone, Debug, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TextSelectionChange {
    pub text_revision: u32,
    pub selection_revision: u32,
    pub search_revision: u32,
    pub selected_paragraphs: u32,
    pub detached: bool,
}

pub fn native_module() -> ModuleDefinition {
    ModuleDefinition::new("solid-gpui-text", "1.0.0", vec![
        ComponentDefinition::element::<TextSelectionObserverProps, _>("TextSelectionObserver", vec![EventDefinition::new::<TextSelectionChange>("selectionChange")], |_, _| gpui::div().w(gpui::px(0.)).h(gpui::px(0.)))
            .with_props(&[]).with_semantic_version("1.0.0"),
    ], vec![
        CommandDefinition::renderer("getTextSelection", |(): (), root, _, _| root.text_selection_snapshot()),
        CommandDefinition::renderer("setTextSelection", |request: TextSelectionRequest, root, _, cx| {
            root.set_text_selection(request)?;
            cx.notify();
            root.text_selection_snapshot()
        }),
        CommandDefinition::renderer("clearTextSelection", |(): (), root, _, cx| {
            root.clear_text_selection();
            cx.notify();
            root.text_selection_snapshot()
        }),
        CommandDefinition::renderer("copyTextSelection", |(): (), root, _, cx| {
            let snapshot = root.text_selection_snapshot()?;
            if !snapshot.text.is_empty() { cx.write_to_clipboard(ClipboardItem::new_string(snapshot.text.clone())); }
            Ok(snapshot)
        }),
        CommandDefinition::renderer("searchText", |request: TextSearchRequest, root, _, cx| {
            let result = root.search_text(request)?;
            cx.notify();
            Ok(result)
        }),
        CommandDefinition::renderer("getTextSearch", |(): (), root, _, _| root.text_search_snapshot()),
        CommandDefinition::renderer("selectTextSearchMatch", |request: TextSearchSelection, root, window, cx| {
            root.select_text_search_match(request, window, cx)
        }),
    ]).with_implementation(include_str!("text.rs"))
        .with_implementation(include_str!("../renderer/selection.rs"))
}
