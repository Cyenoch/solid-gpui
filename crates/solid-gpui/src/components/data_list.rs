//! A native virtualized List delegate with stable keys and explicit load completion.
use super::{ControlSize, choices::Choice};
use crate::native::{
    ControlledBinding, Event, EventDefinition, NativeChildren, NativeView, ViewCommand,
};
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, ScrollStrategy,
    SharedString, Styled, Task, Window, px,
};
use gpui_component::{
    IndexPath, Sizable,
    list::{ListDelegate, ListItem, ListState},
    searchable_list::SearchableListItem,
};
use std::{collections::HashSet, sync::Arc};
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
pub struct ListSection {
    pub key: String,
    #[serde(default)]
    pub header: Option<String>,
    #[serde(default)]
    pub footer: Option<String>,
    pub items: Vec<Choice>,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct ListProps {
    pub sections: Vec<ListSection>,
    pub data_revision: u32,
    pub selected_key: Option<String>,
    pub ack_edit_seq: u32,
    pub query: Option<String>,
    pub searchable: bool,
    pub filterable: bool,
    pub selectable: bool,
    pub loading: bool,
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub row_height: f32,
    pub size: ControlSize,
    pub search_placeholder: Option<String>,
    pub scrollbar_visible: bool,
}
impl Default for ListProps {
    fn default() -> Self {
        Self {
            sections: vec![],
            data_revision: 0,
            selected_key: None,
            ack_edit_seq: 0,
            query: None,
            searchable: false,
            filterable: true,
            selectable: true,
            loading: false,
            has_more: false,
            next_cursor: None,
            row_height: 40.,
            size: ControlSize::Medium,
            search_placeholder: None,
            scrollbar_visible: true,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KeySelection {
    pub value: Option<String>,
    pub edit_seq: u32,
    pub data_revision: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RowConfirmation {
    pub key: String,
    pub secondary: bool,
    pub data_revision: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataQuery {
    pub query: String,
    pub request_id: u32,
    pub data_revision: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageRequest {
    pub request_id: u32,
    pub data_revision: u32,
    pub cursor: Option<String>,
}
#[derive(Default)]
pub(super) struct PaginationGate {
    sequence: u32,
    pending: Option<u32>,
}
impl PaginationGate {
    pub fn request(&mut self, cursor: Option<String>, revision: u32, event: Event<PageRequest>) {
        if self.pending.is_some() {
            return;
        }
        self.sequence = self
            .sequence
            .checked_add(1)
            .expect("page request sequence exhausted");
        self.pending = Some(self.sequence);
        event.emit(PageRequest {
            request_id: self.sequence,
            data_revision: revision,
            cursor,
        });
    }
    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn cancel(&mut self) {
        self.pending = None;
    }
    pub fn finish(&mut self, id: u32) -> Result<(), String> {
        if self.pending != Some(id) {
            return Err("page response does not match the outstanding request".into());
        }
        self.pending = None;
        Ok(())
    }
}
struct Rows {
    data: Arc<Vec<ListSection>>,
    visible: Vec<Vec<usize>>,
    query: String,
    query_id: u32,
    revision: u32,
    generation: u32,
    selected: Option<String>,
    edit_seq: u32,
    events: Event<KeySelection>,
    children: NativeChildren,
    filterable: bool,
    loading: bool,
    has_more: bool,
    next_cursor: Option<String>,
    pager: PaginationGate,
    row_height: f32,
}
impl Rows {
    fn filter(data: &[ListSection], query: &str, enabled: bool) -> Vec<Vec<usize>> {
        let q = query.trim().to_lowercase();
        data.iter()
            .map(|s| {
                s.items
                    .iter()
                    .enumerate()
                    .filter_map(|(i, row)| {
                        (!enabled
                            || q.is_empty()
                            || row.label.to_lowercase().contains(&q)
                            || row.keywords.iter().any(|k| k.to_lowercase().contains(&q)))
                        .then_some(i)
                    })
                    .collect()
            })
            .collect()
    }
    fn row(&self, ix: IndexPath) -> Option<&Choice> {
        let row = *self.visible.get(ix.section)?.get(ix.row)?;
        self.data.get(ix.section)?.items.get(row)
    }
    fn position(&self, key: &str) -> Option<IndexPath> {
        self.visible.iter().enumerate().find_map(|(s, rows)| {
            rows.iter()
                .position(|r| self.data[s].items[*r].key == key)
                .map(|r| IndexPath::default().section(s).row(r))
        })
    }
    fn selected_index(&self) -> Option<IndexPath> {
        self.selected.as_deref().and_then(|k| self.position(k))
    }
}
impl ListDelegate for Rows {
    type Item = ListItem;
    fn sections_count(&self, _: &App) -> usize {
        self.data.len()
    }
    fn items_count(&self, section: usize, _: &App) -> usize {
        self.visible.get(section).map_or(0, Vec::len)
    }
    fn item_key(&self, ix: IndexPath, _: &App) -> Option<SharedString> {
        self.row(ix).map(|r| r.key.clone().into())
    }
    fn index_for_key(&self, key: &str, _: &App) -> Option<IndexPath> {
        self.position(key)
    }
    fn can_select_index(&self, ix: IndexPath, _: &App) -> bool {
        self.row(ix).is_some_and(|r| !r.disabled)
    }
    fn render_item(
        &mut self,
        ix: IndexPath,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<ListItem> {
        let row = self.row(ix)?;
        Some(
            ListItem::new(SharedString::from(row.key.clone()))
                .disabled(row.disabled)
                .h(px(self.row_height))
                .overflow_hidden()
                .child(row.render(window, cx)),
        )
    }
    fn render_section_header(
        &mut self,
        section: usize,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
        self.data
            .get(section)?
            .header
            .as_ref()
            .map(|s| gpui::div().h(px(28.)).px_2().text_sm().child(s.clone()))
    }
    fn render_section_footer(
        &mut self,
        section: usize,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
        self.data
            .get(section)?
            .footer
            .as_ref()
            .map(|s| gpui::div().h(px(28.)).px_2().text_sm().child(s.clone()))
    }
    fn render_empty(
        &mut self,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> impl IntoElement {
        self.children.slot("empty")
    }
    fn render_loading(
        &mut self,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> impl IntoElement {
        let loading = self.children.slot("loading");
        if loading.is_empty() {
            gpui_component::spinner::Spinner::new().into_any_element()
        } else {
            loading.into_any_element()
        }
    }
    fn loading(&self, _: &App) -> bool {
        self.loading
    }
    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) {
        let key = ix.and_then(|i| self.row(i)).map(|r| r.key.clone());
        if key == self.selected {
            return;
        }
        self.selected = key.clone();
        self.edit_seq = self
            .edit_seq
            .checked_add(1)
            .expect("list edit sequence exhausted");
        self.events.emit(KeySelection {
            value: key,
            edit_seq: self.edit_seq,
            data_revision: self.revision,
        });
    }
    fn set_right_clicked_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) {
        self.events
            .related("rightClick")
            .emit(ix.and_then(|i| self.row(i)).map(|r| r.key.clone()));
    }
    fn confirm(&mut self, secondary: bool, _: &mut Window, _: &mut Context<ListState<Self>>) {
        if let Some(key) = &self.selected {
            self.events.related("confirm").emit(RowConfirmation {
                key: key.clone(),
                secondary,
                data_revision: self.revision,
            });
        }
    }
    fn cancel(&mut self, _: &mut Window, _: &mut Context<ListState<Self>>) {
        self.events.related("cancel").emit(());
    }
    fn has_more(&self, _: &App) -> bool {
        self.has_more && !self.pager.is_pending() && !self.loading
    }
    fn load_more(&mut self, _: &mut Window, _: &mut Context<ListState<Self>>) {
        if self.has_more && !self.loading {
            self.pager.request(
                self.next_cursor.clone(),
                self.revision,
                self.events.related("loadMore"),
            );
        }
    }
    fn perform_search(
        &mut self,
        query: &str,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.query = query.into();
        self.query_id = self
            .query_id
            .checked_add(1)
            .expect("list query sequence exhausted");
        self.pager.cancel();
        self.events.related("query").emit(DataQuery {
            query: query.into(),
            request_id: self.query_id,
            data_revision: self.revision,
        });
        let query = self.query.clone();
        let generation = self.generation;
        let query_id = self.query_id;
        let filterable = self.filterable;
        let data = self.data.clone();
        let filter = cx
            .background_executor()
            .spawn(async move { Self::filter(&data, &query, filterable) });
        cx.spawn(async move |weak, cx| {
            let visible = filter.await;
            let _ = weak.update(cx, |state, cx| {
                let rows = state.delegate_mut();
                if rows.generation != generation || rows.query_id != query_id {
                    return;
                }
                rows.visible = visible;
                let index = rows.selected_index();
                state.project_selection(index, cx);
            });
        })
    }
}
pub struct List {
    state: Entity<ListState<Rows>>,
    props: ListProps,
}
#[crate::component]
impl NativeView for List {
    type Props = ListProps;
    type Event = KeySelection;
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        if p.sections.len() > 1000
            || p.sections.iter().map(|s| s.items.len()).sum::<usize>() > 1000000
            || !p.row_height.is_finite()
            || !(16. ..=512.).contains(&p.row_height)
        {
            return Err(
                "list requires at most 1000 sections, 1000000 rows and rowHeight 16..512".into(),
            );
        }
        let mut groups = HashSet::new();
        let mut keys = HashSet::new();
        for s in &p.sections {
            if s.key.is_empty() || !groups.insert(&s.key) {
                return Err("list section keys must be nonempty and unique".into());
            }
            for row in &s.items {
                if row.key.is_empty() || !keys.insert(&row.key) {
                    return Err("list row keys must be globally unique and nonempty".into());
                }
            }
        }
        if p.selected_key.as_ref().is_some_and(|k| !keys.contains(k)) {
            return Err("selectedKey must exist in list sections".into());
        }
        Ok(())
    }
    fn slots() -> &'static [&'static str] {
        &["empty", "loading"]
    }
    fn event_name() -> &'static str {
        "select"
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![
            EventDefinition::new::<RowConfirmation>("confirm"),
            EventDefinition::new::<Option<String>>("rightClick"),
            EventDefinition::new::<()>("cancel"),
            EventDefinition::new::<DataQuery>("query"),
            EventDefinition::new::<PageRequest>("loadMore"),
        ]
    }
    fn controlled() -> Option<ControlledBinding> {
        Some(ControlledBinding {
            value_prop: "selectedKey",
            event_id: 1,
            sequence_field: "editSeq",
            ack_prop: "ackEditSeq",
        })
    }
    fn mount(
        props: Self::Props,
        events: Event<KeySelection>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let data = Arc::new(props.sections.clone());
        let query = props.query.clone().unwrap_or_default();
        let visible = Rows::filter(&data, &query, props.filterable);
        let rows = Rows {
            data,
            visible,
            query,
            query_id: 0,
            revision: props.data_revision,
            generation: 0,
            selected: props.selected_key.clone(),
            edit_seq: 0,
            events,
            children,
            filterable: props.filterable,
            loading: props.loading,
            has_more: props.has_more,
            next_cursor: props.next_cursor.clone(),
            pager: PaginationGate::default(),
            row_height: props.row_height,
        };
        let index = rows.selected_index();
        let state = cx.new(|cx| {
            let mut s = ListState::new(rows, window, cx)
                .searchable(props.searchable)
                .search_selects_first(false)
                .selectable(props.selectable);
            s.project_selection(index, cx);
            if let Some(query) = &props.query {
                s.set_query(query, window, cx);
            }
            s
        });
        Self { state, props }
    }
    fn update(&mut self, p: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            let rows = s.delegate_mut();
            let data_changed =
                p.sections != self.props.sections || p.filterable != self.props.filterable;
            let selection_changed = (p.selected_key != self.props.selected_key
                || p.ack_edit_seq != self.props.ack_edit_seq)
                && (self.props.selected_key.is_none() && p.selected_key.is_some()
                    || p.ack_edit_seq >= rows.edit_seq);
            if data_changed {
                rows.generation = rows
                    .generation
                    .checked_add(1)
                    .expect("list data generation exhausted");
                rows.data = Arc::new(p.sections.clone());
                rows.visible = Rows::filter(&rows.data, &rows.query, p.filterable);
                rows.filterable = p.filterable;
            }
            if selection_changed {
                rows.selected = p.selected_key.clone();
            } else if rows.selected.as_ref().is_some_and(|k| {
                !rows
                    .data
                    .iter()
                    .any(|s| s.items.iter().any(|r| &r.key == k))
            }) {
                rows.selected = None;
            }
            rows.revision = p.data_revision;
            rows.loading = p.loading;
            rows.has_more = p.has_more;
            rows.next_cursor = p.next_cursor.clone();
            rows.row_height = p.row_height;
            let index = rows.selected_index();
            if data_changed || selection_changed {
                s.project_selection(index, cx);
            }
            if p.searchable != self.props.searchable {
                s.set_searchable(p.searchable, cx);
            }
            if p.selectable != self.props.selectable {
                s.set_selectable(p.selectable, cx);
            }
            if p.query != self.props.query
                && let Some(query) = &p.query
            {
                s.set_query(query, window, cx);
            }
            cx.notify();
        });
        self.props = p;
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("focus", |this, (): (), window, cx| {
                this.state.update(cx, |s, cx| s.focus(window, cx));
                Ok(())
            }),
            ViewCommand::new("scrollTo", |this, key: String, window, cx| {
                this.state.update(cx, |s, cx| {
                    let index = s
                        .delegate()
                        .position(&key)
                        .ok_or("scrollTo key is not visible in the current query")?;
                    s.scroll_to_item(index, ScrollStrategy::Center, window, cx);
                    Ok(())
                })
            }),
            ViewCommand::new("finishLoad", |this, request_id: u32, _, cx| {
                this.state.update(cx, |s, cx| {
                    s.delegate_mut().pager.finish(request_id)?;
                    cx.notify();
                    Ok(())
                })
            }),
            ViewCommand::new("setQuery", |this, query: String, window, cx| {
                this.state
                    .update(cx, |s, cx| s.set_query(&query, window, cx));
                Ok(())
            }),
        ]
    }
}
impl Render for List {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut v = gpui_component::list::List::new(&self.state)
            .with_size(self.props.size)
            .scrollbar_visible(self.props.scrollbar_visible);
        if let Some(text) = &self.props.search_placeholder {
            v = v.search_placeholder(text.clone());
        }
        v
    }
}
pub(crate) fn definition() -> crate::native::ComponentDefinition {
    __native_component_List()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    fn sections(reverse: bool) -> Vec<ListSection> {
        let mut rows: Vec<Choice> = (0..100)
            .map(|i| Choice {
                key: format!("k{i}"),
                label: format!("Row {i}"),
                keywords: vec![],
                disabled: i == 1,
                description: None,
                icon: None,
            })
            .collect();
        if reverse {
            rows.reverse();
        }
        vec![ListSection {
            key: "rows".into(),
            header: None,
            footer: None,
            items: rows,
        }]
    }
    #[gpui::test]
    fn search_and_reorder_keep_hidden_selection_and_load_request_ownership(
        cx: &mut gpui::TestAppContext,
    ) {
        let fixture = Fixture::<List>::new(
            ListProps {
                sections: sections(false),
                selected_key: Some("k2".into()),
                searchable: true,
                has_more: true,
                ..Default::default()
            },
            cx,
        );
        fixture.update(cx, |v, window, cx| {
            let entity = v.state.entity_id();
            v.state.update(cx, |s, cx| s.set_query("Row 7", window, cx));
            v.update(
                ListProps {
                    sections: sections(true),
                    selected_key: Some("k2".into()),
                    searchable: true,
                    has_more: true,
                    data_revision: 1,
                    ..Default::default()
                },
                window,
                cx,
            );
            assert_eq!(v.state.entity_id(), entity);
        });
        cx.run_until_parked();
        fixture.update(cx, |v, window, cx| {
            v.state.update(cx, |s, cx| {
                assert_eq!(s.delegate().selected.as_deref(), Some("k2"));
                assert_eq!(s.selected_index(), None);
                let rows = s.delegate_mut();
                rows.load_more(window, cx);
                rows.load_more(window, cx);
                assert!(rows.pager.finish(2).is_err());
                assert!(rows.pager.is_pending());
                rows.pager.finish(1).unwrap();
                rows.load_more(window, cx);
                assert!(rows.pager.finish(1).is_err());
                rows.pager.finish(2).unwrap();
            });
        });
        let mut loads = 0;
        let mut selections = 0;
        while let Some(event) = fixture.runtime.take_event().unwrap() {
            if let crate::EventPayload::Extension { event_id, .. } = event.payload {
                if event_id == 1 {
                    selections += 1;
                }
                if event_id == 4 {
                    loads += 1;
                }
            }
        }
        assert_eq!(
            selections, 0,
            "prop projection and filtering must not synthesize selection events"
        );
        assert_eq!(loads, 2, "only one outstanding page request is allowed");
    }
}
