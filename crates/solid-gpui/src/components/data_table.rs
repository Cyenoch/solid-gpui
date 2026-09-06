//! Native DataTable delegates keep data, keys and sorting on the Rust side.
use super::{
    ControlSize,
    data_list::{PageRequest, PaginationGate},
    table_elements::CellAlign,
};
use crate::native::{
    ControlledBinding, Event, EventDefinition, NativeChildren, NativeView, ViewCommand,
};
use gpui::{
    App, AppContext, Context, Entity, Focusable, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, Styled, Task, Window, div, px,
};
use gpui_component::{
    Sizable,
    table::{Column, ColumnGroup, ColumnSort, TableDelegate, TableEvent, TableState},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ops::Range,
    sync::Arc,
};

#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    None,
    Ascending,
    Descending,
}
impl From<SortOrder> for ColumnSort {
    fn from(v: SortOrder) -> Self {
        match v {
            SortOrder::None => Self::Default,
            SortOrder::Ascending => Self::Ascending,
            SortOrder::Descending => Self::Descending,
        }
    }
}
impl From<ColumnSort> for SortOrder {
    fn from(v: ColumnSort) -> Self {
        match v {
            ColumnSort::Default => Self::None,
            ColumnSort::Ascending => Self::Ascending,
            ColumnSort::Descending => Self::Descending,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TableColumn {
    pub key: String,
    pub label: String,
    #[serde(default = "column_width")]
    pub width: f32,
    #[serde(default = "column_min")]
    pub min_width: f32,
    #[serde(default = "column_max")]
    pub max_width: f32,
    #[serde(default)]
    pub align: CellAlign,
    #[serde(default)]
    pub fixed: bool,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub sort: SortOrder,
    #[serde(default = "yes")]
    pub resizable: bool,
    #[serde(default = "yes")]
    pub movable: bool,
    #[serde(default = "yes")]
    pub selectable: bool,
}
fn yes() -> bool {
    true
}
fn column_width() -> f32 {
    160.
}
fn column_min() -> f32 {
    24.
}
fn column_max() -> f32 {
    4096.
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum TableValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Badge { badge: String },
    Icon { icon: String, label: String },
    Progress { progress: f32 },
}
impl TableValue {
    fn text(&self) -> String {
        match self {
            Self::Text(v) => v.clone(),
            Self::Number(v) => v.to_string(),
            Self::Bool(v) => v.to_string(),
            Self::Badge { badge } => badge.clone(),
            Self::Icon { label, .. } => label.clone(),
            Self::Progress { progress } => format!("{progress}%"),
        }
    }
    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.total_cmp(b),
            (Self::Progress { progress: a }, Self::Progress { progress: b }) => a.total_cmp(b),
            (Self::Bool(a), Self::Bool(b)) => a.cmp(b),
            _ => self.text().cmp(&other.text()),
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
pub struct TableRowData {
    pub key: String,
    pub cells: BTreeMap<String, TableValue>,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
pub struct TableHeaderGroup {
    pub label: String,
    pub span: u16,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TableSelection {
    Row { row_key: String },
    Column { column_key: String },
    Cell { row_key: String, column_key: String },
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct DataTableProps {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRowData>,
    pub group_headers: Vec<Vec<TableHeaderGroup>>,
    pub selection: Option<TableSelection>,
    pub ack_edit_seq: u32,
    pub data_revision: u32,
    pub manual_sort: bool,
    pub row_selectable: bool,
    pub column_selectable: bool,
    pub cell_selectable: bool,
    pub row_header: bool,
    pub loop_selection: bool,
    pub loading: bool,
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub stripe: bool,
    pub bordered: bool,
    pub size: ControlSize,
    pub vertical_scrollbar: bool,
    pub horizontal_scrollbar: bool,
}
impl Default for DataTableProps {
    fn default() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            group_headers: vec![],
            selection: None,
            ack_edit_seq: 0,
            data_revision: 0,
            manual_sort: false,
            row_selectable: true,
            column_selectable: true,
            cell_selectable: false,
            row_header: true,
            loop_selection: true,
            loading: false,
            has_more: false,
            next_cursor: None,
            stripe: false,
            bordered: true,
            size: ControlSize::Medium,
            vertical_scrollbar: true,
            horizontal_scrollbar: true,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TableSelectionChange {
    pub value: Option<TableSelection>,
    pub edit_seq: u32,
    pub data_revision: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TableSortChange {
    pub column_key: String,
    pub order: SortOrder,
    pub data_revision: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TableColumnMove {
    pub column_key: String,
    pub from: u32,
    pub to: u32,
    pub keys: Vec<String>,
}
#[crate::native_type]
#[derive(Clone, Debug)]
pub struct TableColumnWidth {
    pub key: String,
    pub width: f32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
pub struct TableRange {
    pub start: u32,
    pub end: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
pub struct TableExport {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}
struct Data {
    rows: Vec<TableRowData>,
    by_key: HashMap<String, usize>,
}
impl Data {
    fn new(rows: Vec<TableRowData>) -> Self {
        Self {
            by_key: rows
                .iter()
                .enumerate()
                .map(|(i, r)| (r.key.clone(), i))
                .collect(),
            rows,
        }
    }
}
struct Rows {
    data: Arc<Data>,
    order: Vec<usize>,
    positions: HashMap<String, usize>,
    columns: Vec<TableColumn>,
    groups: Vec<Vec<TableHeaderGroup>>,
    selection: Option<TableSelection>,
    edit_seq: u32,
    revision: u32,
    generation: u32,
    manual_sort: bool,
    loading: bool,
    has_more: bool,
    cursor: Option<String>,
    pager: PaginationGate,
    events: Event<TableSelectionChange>,
    children: NativeChildren,
    sort_task: Task<()>,
}
impl Rows {
    fn row(&self, index: usize) -> Option<&TableRowData> {
        self.order.get(index).and_then(|i| self.data.rows.get(*i))
    }
    fn cell(&self, row: usize, col: usize) -> Option<&TableValue> {
        self.row(row)?.cells.get(&self.columns.get(col)?.key)
    }
    fn rebuild_positions(&mut self) {
        self.positions = self
            .order
            .iter()
            .enumerate()
            .map(|(i, r)| (self.data.rows[*r].key.clone(), i))
            .collect();
    }
    fn selection_indices(&self) -> (Option<usize>, Option<usize>, Option<(usize, usize)>) {
        let row = |key: &String| self.positions.get(key).copied();
        let col = |key: &String| self.columns.iter().position(|c| &c.key == key);
        match &self.selection {
            Some(TableSelection::Row { row_key }) => (row(row_key), None, None),
            Some(TableSelection::Column { column_key }) => (None, col(column_key), None),
            Some(TableSelection::Cell {
                row_key,
                column_key,
            }) => (None, None, row(row_key).zip(col(column_key))),
            None => (None, None, None),
        }
    }
    fn valid_selection(&self, s: &TableSelection) -> bool {
        match s {
            TableSelection::Row { row_key } => self.data.by_key.contains_key(row_key),
            TableSelection::Column { column_key } => {
                self.columns.iter().any(|c| &c.key == column_key)
            }
            TableSelection::Cell {
                row_key,
                column_key,
            } => {
                self.data.by_key.contains_key(row_key)
                    && self.columns.iter().any(|c| &c.key == column_key)
            }
        }
    }
    fn select(&mut self, next: Option<TableSelection>) {
        if self.selection == next {
            return;
        }
        self.selection = next.clone();
        self.edit_seq = self
            .edit_seq
            .checked_add(1)
            .expect("table edit sequence exhausted");
        self.events.emit(TableSelectionChange {
            value: next,
            edit_seq: self.edit_seq,
            data_revision: self.revision,
        });
    }
    fn event_selection(&self, row: Option<usize>, col: Option<usize>) -> Option<TableSelection> {
        match (row, col) {
            (Some(r), Some(c)) => Some(TableSelection::Cell {
                row_key: self.row(r)?.key.clone(),
                column_key: self.columns.get(c)?.key.clone(),
            }),
            (Some(r), None) => Some(TableSelection::Row {
                row_key: self.row(r)?.key.clone(),
            }),
            (None, Some(c)) => Some(TableSelection::Column {
                column_key: self.columns.get(c)?.key.clone(),
            }),
            _ => None,
        }
    }
    fn sort(&mut self, window: &mut Window, cx: &mut Context<TableState<Self>>) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("table sort generation exhausted");
        let generation = self.generation;
        let data = self.data.clone();
        let sorted = self
            .columns
            .iter()
            .find(|c| c.sort != SortOrder::None)
            .map(|c| (c.key.clone(), c.sort));
        let task = cx.background_executor().spawn(async move {
            let mut order: Vec<_> = (0..data.rows.len()).collect();
            if let Some((key, direction)) = sorted {
                order.sort_by(|a, b| {
                    let cmp = match (data.rows[*a].cells.get(&key), data.rows[*b].cells.get(&key)) {
                        (Some(a), Some(b)) => a.compare(b),
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (None, None) => std::cmp::Ordering::Equal,
                    };
                    if direction == SortOrder::Descending {
                        cmp.reverse()
                    } else {
                        cmp
                    }
                });
            }
            order
        });
        self.sort_task = cx.spawn_in(window, async move |weak, cx| {
            let order = task.await;
            let _ = weak.update_in(cx, |state, _, cx| {
                let rows = state.delegate_mut();
                if rows.generation != generation {
                    return;
                }
                rows.order = order;
                rows.rebuild_positions();
                let (r, c, cell) = rows.selection_indices();
                state.project_selection(r, c, cell, cx);
            });
        });
    }
}
impl TableDelegate for Rows {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }
    fn rows_count(&self, _: &App) -> usize {
        self.order.len()
    }
    fn row_key(&self, row: usize, _: &App) -> Option<SharedString> {
        self.row(row).map(|r| r.key.clone().into())
    }
    fn row_index(&self, key: &str, _: &App) -> Option<usize> {
        self.positions.get(key).copied()
    }
    fn column(&self, index: usize, _: &App) -> Column {
        let c = &self.columns[index];
        let mut column = Column::new(c.key.clone(), c.label.clone())
            .width(px(c.width))
            .min_width(px(c.min_width))
            .max_width(px(c.max_width))
            .resizable(c.resizable)
            .movable(c.movable)
            .selectable(c.selectable);
        column.align = match c.align {
            CellAlign::Left => gpui::TextAlign::Left,
            CellAlign::Center => gpui::TextAlign::Center,
            CellAlign::Right => gpui::TextAlign::Right,
        };
        if c.fixed {
            column = column.fixed_left();
        }
        if c.sortable {
            column = column.sort(c.sort.into());
        }
        column
    }
    fn group_headers(&self, _: &App) -> Option<Vec<Vec<ColumnGroup>>> {
        (!self.groups.is_empty()).then(|| {
            self.groups
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|g| ColumnGroup::new(g.label.clone(), g.span as usize))
                        .collect()
                })
                .collect()
        })
    }
    fn render_tr(
        &mut self,
        row: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> gpui::Stateful<gpui::Div> {
        div().id(self
            .row(row)
            .map_or_else(|| format!("filler-{row}"), |r| r.key.clone()))
    }
    fn render_td(
        &mut self,
        row: usize,
        col: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        match self.cell(row, col) {
            Some(TableValue::Badge { badge }) => gpui_component::tag::Tag::new()
                .child(badge.clone())
                .into_any_element(),
            Some(TableValue::Icon { icon, label }) => gpui_component::h_flex()
                .gap_2()
                .child(gpui_component::Icon::default().path(icon.clone()).size_4())
                .child(label.clone())
                .into_any_element(),
            Some(TableValue::Progress { progress }) => {
                gpui_component::progress::Progress::new(SharedString::from(format!(
                    "cell-progress:{}:{}",
                    self.row(row).unwrap().key,
                    self.columns[col].key
                )))
                .value(*progress)
                .into_any_element()
            }
            Some(v) => div().overflow_hidden().child(v.text()).into_any_element(),
            None => gpui::Empty.into_any_element(),
        }
    }
    fn cell_text(&self, row: usize, col: usize, _: &App) -> String {
        self.cell(row, col)
            .map_or_else(String::new, TableValue::text)
    }
    fn perform_sort(
        &mut self,
        col: usize,
        sort: ColumnSort,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        let key = self.columns[col].key.clone();
        for (i, c) in self.columns.iter_mut().enumerate() {
            c.sort = if i == col {
                sort.into()
            } else {
                SortOrder::None
            };
        }
        self.events.related("sort").emit(TableSortChange {
            column_key: key,
            order: sort.into(),
            data_revision: self.revision,
        });
        self.pager.cancel();
        if !self.manual_sort {
            self.sort(window, cx);
        }
    }
    fn move_column(
        &mut self,
        from: usize,
        to: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
        let column = self.columns.remove(from);
        let key = column.key.clone();
        self.columns.insert(to, column);
        self.events.related("moveColumn").emit(TableColumnMove {
            column_key: key,
            from: from as u32,
            to: to as u32,
            keys: self.columns.iter().map(|c| c.key.clone()).collect(),
        });
    }
    fn on_event(&mut self, event: &TableEvent, _: &mut Context<TableState<Self>>) {
        match event {
            TableEvent::SelectRow(r) => self.select(self.event_selection(Some(*r), None)),
            TableEvent::SelectColumn(c) => self.select(self.event_selection(None, Some(*c))),
            TableEvent::SelectCell(r, c) => self.select(self.event_selection(Some(*r), Some(*c))),
            TableEvent::ClearSelection => self.select(None),
            TableEvent::DoubleClickedRow(r) => self
                .events
                .related("doubleClick")
                .emit(self.event_selection(Some(*r), None)),
            TableEvent::DoubleClickedCell(r, c) => self
                .events
                .related("doubleClick")
                .emit(self.event_selection(Some(*r), Some(*c))),
            TableEvent::RightClickedRow(r) => self
                .events
                .related("rightClick")
                .emit(self.event_selection(*r, None)),
            TableEvent::RightClickedCell(r, c) => self
                .events
                .related("rightClick")
                .emit(self.event_selection(Some(*r), Some(*c))),
            TableEvent::ColumnWidthsChanged(widths) => self.events.related("columnWidths").emit(
                self.columns
                    .iter()
                    .zip(widths)
                    .map(|(c, w)| TableColumnWidth {
                        key: c.key.clone(),
                        width: f32::from(*w),
                    })
                    .collect::<Vec<_>>(),
            ),
            TableEvent::MoveColumn(..) => {}
        }
    }
    fn loading(&self, _: &App) -> bool {
        self.loading
    }
    fn render_empty(
        &mut self,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        self.children.slot("empty")
    }
    fn render_loading(
        &mut self,
        _: gpui_component::Size,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let slot = self.children.slot("loading");
        if slot.is_empty() {
            gpui_component::spinner::Spinner::new().into_any_element()
        } else {
            slot.into_any_element()
        }
    }
    fn has_more(&self, _: &App) -> bool {
        self.has_more && !self.loading && !self.pager.is_pending()
    }
    fn load_more(&mut self, _: &mut Window, _: &mut Context<TableState<Self>>) {
        if self.has_more && !self.loading {
            self.pager.request(
                self.cursor.clone(),
                self.revision,
                self.events.related("loadMore"),
            );
        }
    }
    fn visible_rows_changed(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
        self.events.related("visibleRows").emit(TableRange {
            start: range.start as u32,
            end: range.end as u32,
        });
    }
    fn visible_columns_changed(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
        self.events.related("visibleColumns").emit(TableRange {
            start: range.start as u32,
            end: range.end as u32,
        });
    }
}
pub struct DataTable {
    state: Entity<TableState<Rows>>,
    props: DataTableProps,
}
impl DataTable {
    fn configure(state: &mut TableState<Rows>, p: &DataTableProps) {
        state.row_selectable = p.row_selectable;
        state.col_selectable = p.column_selectable;
        state.cell_selectable = p.cell_selectable;
        state.row_header = p.row_header;
        state.loop_selection = p.loop_selection;
    }
}
#[crate::component]
impl NativeView for DataTable {
    type Props = DataTableProps;
    type Event = TableSelectionChange;
    fn event_name() -> &'static str {
        "selectionChange"
    }
    fn slots() -> &'static [&'static str] {
        &["empty", "loading"]
    }
    fn controlled() -> Option<ControlledBinding> {
        Some(ControlledBinding {
            value_prop: "selection",
            event_id: 1,
            sequence_field: "editSeq",
            ack_prop: "ackEditSeq",
        })
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![
            EventDefinition::new::<TableSortChange>("sort"),
            EventDefinition::new::<TableColumnMove>("moveColumn"),
            EventDefinition::new::<Vec<TableColumnWidth>>("columnWidths"),
            EventDefinition::new::<Option<TableSelection>>("doubleClick"),
            EventDefinition::new::<Option<TableSelection>>("rightClick"),
            EventDefinition::new::<PageRequest>("loadMore"),
            EventDefinition::new::<TableRange>("visibleRows"),
            EventDefinition::new::<TableRange>("visibleColumns"),
        ]
    }
    fn validate_props(p: &DataTableProps) -> Result<(), String> {
        if p.columns.len() > 256 || p.rows.len() > 1000000 || p.group_headers.len() > 8 {
            return Err(
                "table supports at most 256 columns, 1000000 rows and 8 header levels".into(),
            );
        }
        let mut cols = HashSet::new();
        let mut keys = HashSet::new();
        let mut unpinned = false;
        let mut sorts = 0;
        for c in &p.columns {
            if c.key.is_empty() || !cols.insert(c.key.as_str()) {
                return Err("table column keys must be nonempty and unique".into());
            }
            if ![c.width, c.min_width, c.max_width]
                .into_iter()
                .all(f32::is_finite)
                || c.min_width < 8.
                || c.max_width > 16384.
                || c.min_width > c.width
                || c.width > c.max_width
            {
                return Err(
                    "table columns require 8 <= minWidth <= width <= maxWidth <= 16384".into(),
                );
            }
            if !c.fixed {
                unpinned = true;
            } else if unpinned {
                return Err("fixed columns must be contiguous at the left".into());
            }
            if c.sort != SortOrder::None {
                sorts += 1;
                if !c.sortable {
                    return Err("a sorted column must be sortable".into());
                }
            }
        }
        if sorts > 1 {
            return Err("table supports one sorted column".into());
        }
        for row in &p.rows {
            if row.key.is_empty() || !keys.insert(row.key.as_str()) {
                return Err("table row keys must be nonempty and unique".into());
            }
            for (key, value) in &row.cells {
                if !cols.contains(key.as_str()) {
                    return Err("table cell key must name a declared column".into());
                }
                if let TableValue::Progress { progress } = value
                    && (!progress.is_finite() || !(0. ..=100.).contains(progress))
                {
                    return Err("table progress must be 0..100".into());
                }
            }
        }
        for level in &p.group_headers {
            if level.iter().any(|g| g.span == 0)
                || level.iter().map(|g| g.span as usize).sum::<usize>() != p.columns.len()
            {
                return Err("each table header level must span exactly all columns".into());
            }
        }
        if let Some(s) = &p.selection {
            let (r, c) = match s {
                TableSelection::Row { row_key } => (Some(row_key), None),
                TableSelection::Column { column_key } => (None, Some(column_key)),
                TableSelection::Cell {
                    row_key,
                    column_key,
                } => (Some(row_key), Some(column_key)),
            };
            if r.is_some_and(|k| !keys.contains(k.as_str()))
                || c.is_some_and(|k| !cols.contains(k.as_str()))
            {
                return Err("table selection must reference existing row and column keys".into());
            }
        }
        Ok(())
    }
    fn mount(
        props: DataTableProps,
        events: Event<TableSelectionChange>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let data = Arc::new(Data::new(props.rows.clone()));
        let mut rows = Rows {
            order: (0..data.rows.len()).collect(),
            data,
            positions: HashMap::new(),
            columns: props.columns.clone(),
            groups: props.group_headers.clone(),
            selection: props.selection.clone(),
            edit_seq: 0,
            revision: props.data_revision,
            generation: 0,
            manual_sort: props.manual_sort,
            loading: props.loading,
            has_more: props.has_more,
            cursor: props.next_cursor.clone(),
            pager: PaginationGate::default(),
            events,
            children,
            sort_task: Task::ready(()),
        };
        rows.rebuild_positions();
        let state = cx.new(|cx| {
            let mut state = TableState::new(rows, window, cx);
            Self::configure(&mut state, &props);
            let (r, c, cell) = state.delegate().selection_indices();
            state.project_selection(r, c, cell, cx);
            if !props.manual_sort {
                state.delegate_mut().sort(window, cx);
            }
            state
        });
        Self { state, props }
    }
    fn update(&mut self, p: DataTableProps, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            let data_changed = p.rows != self.props.rows;
            let columns_changed = p.columns != self.props.columns;
            let rows = state.delegate_mut();
            if data_changed {
                rows.data = Arc::new(Data::new(p.rows.clone()));
                rows.order = (0..rows.data.rows.len()).collect();
                rows.rebuild_positions();
            }
            if columns_changed {
                let old_props: HashMap<_, _> =
                    self.props.columns.iter().map(|c| (&c.key, c)).collect();
                let old_runtime: HashMap<_, _> = rows
                    .columns
                    .iter()
                    .map(|c| (c.key.clone(), c.clone()))
                    .collect();
                let order_changed = p.columns.iter().map(|c| &c.key).ne(self
                    .props
                    .columns
                    .iter()
                    .map(|c| &c.key));
                let sort_changed = p
                    .columns
                    .iter()
                    .any(|c| old_props.get(&c.key).is_none_or(|old| old.sort != c.sort));
                let fixed_changed = p
                    .columns
                    .iter()
                    .any(|c| old_props.get(&c.key).is_none_or(|old| old.fixed != c.fixed));
                let mut columns = p.columns.clone();
                for c in &mut columns {
                    if !sort_changed
                        && c.sortable
                        && let Some(old) = old_runtime.get(&c.key)
                    {
                        c.sort = old.sort;
                    }
                }
                if !order_changed && !fixed_changed {
                    let positions: HashMap<_, _> = rows
                        .columns
                        .iter()
                        .enumerate()
                        .map(|(i, c)| (c.key.clone(), i))
                        .collect();
                    columns.sort_by_key(|c| positions.get(&c.key).copied().unwrap_or(usize::MAX));
                }
                rows.columns = columns;
            }
            if (p.selection != self.props.selection || p.ack_edit_seq != self.props.ack_edit_seq)
                && (self.props.selection.is_none() && p.selection.is_some()
                    || p.ack_edit_seq >= rows.edit_seq)
            {
                rows.selection = p.selection.clone();
            }
            if rows
                .selection
                .as_ref()
                .is_some_and(|s| !rows.valid_selection(s))
            {
                rows.selection = None;
            }
            rows.groups = p.group_headers.clone();
            rows.revision = p.data_revision;
            rows.loading = p.loading;
            rows.has_more = p.has_more;
            rows.cursor = p.next_cursor.clone();
            rows.manual_sort = p.manual_sort;
            if data_changed || columns_changed || p.manual_sort != self.props.manual_sort {
                if p.manual_sort {
                    rows.generation = rows
                        .generation
                        .checked_add(1)
                        .expect("table generation exhausted");
                    rows.sort_task = Task::ready(());
                } else {
                    rows.sort(window, cx);
                }
            }
            let (r, c, cell) = rows.selection_indices();
            if columns_changed || p.group_headers != self.props.group_headers {
                state.refresh(cx);
            }
            state.project_selection(r, c, cell, cx);
            Self::configure(state, &p);
            cx.notify();
        });
        self.props = p;
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("focus", |this, (): (), window, cx| {
                this.state.focus_handle(cx).focus(window, cx);
                Ok(())
            }),
            ViewCommand::new("scrollToRow", |this, key: String, _, cx| {
                this.state.update(cx, |state, cx| {
                    let row = state
                        .delegate()
                        .positions
                        .get(&key)
                        .copied()
                        .ok_or("table row key does not exist")?;
                    state.scroll_to_row(row, cx);
                    Ok(())
                })
            }),
            ViewCommand::new("scrollToColumn", |this, key: String, _, cx| {
                this.state.update(cx, |state, cx| {
                    let col = state
                        .delegate()
                        .columns
                        .iter()
                        .position(|c| c.key == key)
                        .ok_or("table column key does not exist")?;
                    state.scroll_to_col(col, cx);
                    Ok(())
                })
            }),
            ViewCommand::new("finishLoad", |this, request_id: u32, _, cx| {
                this.state.update(cx, |state, cx| {
                    state.delegate_mut().pager.finish(request_id)?;
                    cx.notify();
                    Ok(())
                })
            }),
            ViewCommand::new(
                "setColumnWidths",
                |this, widths: Vec<TableColumnWidth>, _, cx| {
                    this.state.update(cx, |state, cx| {
                        let mut keys = HashSet::new();
                        for w in &widths {
                            let c = state
                                .delegate()
                                .columns
                                .iter()
                                .find(|c| c.key == w.key)
                                .ok_or("table column key does not exist")?;
                            if !keys.insert(&w.key)
                                || !w.width.is_finite()
                                || !(c.min_width..=c.max_width).contains(&w.width)
                            {
                                return Err(
                                    "column widths must be unique and inside declared bounds"
                                        .into(),
                                );
                            }
                        }
                        for w in widths {
                            assert!(state.set_column_width(&w.key, px(w.width), cx));
                        }
                        Ok(())
                    })
                },
            ),
            ViewCommand::new("getColumnWidths", |this, (): (), _, cx| {
                Ok(this
                    .state
                    .read(cx)
                    .column_widths()
                    .into_iter()
                    .map(|(key, width)| TableColumnWidth {
                        key: key.to_string(),
                        width: f32::from(width),
                    })
                    .collect::<Vec<_>>())
            }),
            ViewCommand::new("exportRows", |this, range: TableRange, _, cx| {
                if range.end < range.start || range.end - range.start > 1000 {
                    return Err("exportRows requires an ordered range of at most 1000 rows".into());
                }
                let (headers, rows) = this
                    .state
                    .read(cx)
                    .dump_range(range.start as usize..range.end as usize, cx);
                Ok(TableExport { headers, rows })
            }),
        ]
    }
}
impl Render for DataTable {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui_component::table::DataTable::new(&self.state)
            .with_size(self.props.size)
            .stripe(self.props.stripe)
            .bordered(self.props.bordered)
            .scrollbar_visible(
                self.props.vertical_scrollbar,
                self.props.horizontal_scrollbar,
            )
    }
}
pub(crate) fn definition() -> crate::native::ComponentDefinition {
    __native_component_DataTable()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    fn columns() -> Vec<TableColumn> {
        vec![TableColumn {
            key: "score".into(),
            label: "Score".into(),
            width: 160.,
            min_width: 24.,
            max_width: 4096.,
            align: CellAlign::Right,
            fixed: false,
            sortable: true,
            sort: SortOrder::None,
            resizable: true,
            movable: true,
            selectable: true,
        }]
    }
    fn rows(reverse: bool) -> Vec<TableRowData> {
        let mut r = vec![
            TableRowData {
                key: "a".into(),
                cells: [("score".into(), TableValue::Number(2.))].into(),
            },
            TableRowData {
                key: "b".into(),
                cells: [("score".into(), TableValue::Number(1.))].into(),
            },
        ];
        if reverse {
            r.reverse();
        }
        r
    }
    #[gpui::test]
    fn table_reconciles_width_and_selection_by_key_across_pending_sort_and_data_updates(
        cx: &mut gpui::TestAppContext,
    ) {
        let selected = Some(TableSelection::Cell {
            row_key: "a".into(),
            column_key: "score".into(),
        });
        let fixture = Fixture::<DataTable>::new(
            DataTableProps {
                columns: columns(),
                rows: rows(false),
                selection: selected.clone(),
                cell_selectable: true,
                ..Default::default()
            },
            cx,
        );
        fixture.update(cx, |v, window, cx| {
            let id = v.state.entity_id();
            v.state.update(cx, |state, cx| {
                assert!(state.set_column_width("score", px(280.), cx));
                state
                    .delegate_mut()
                    .perform_sort(0, ColumnSort::Ascending, window, cx);
            });
            let mut columns = columns();
            columns[0].label = "Score updated".into();
            v.update(
                DataTableProps {
                    columns,
                    rows: rows(true),
                    selection: selected.clone(),
                    cell_selectable: true,
                    data_revision: 2,
                    ..Default::default()
                },
                window,
                cx,
            );
            assert_eq!(v.state.entity_id(), id);
            assert_eq!(v.state.read(cx).column_widths()[0].1, px(280.));
        });
        cx.run_until_parked();
        fixture.update(cx, |v, _, cx| {
            let state = v.state.read(cx);
            assert_eq!(state.delegate().row(0).unwrap().key, "b");
            assert_eq!(state.selected_cell(), Some((1, 0)));
            assert_eq!(state.delegate().selection, selected);
            assert_eq!(state.column_widths()[0].1, px(280.));
        });
        while let Some(event) = fixture.runtime.take_event().unwrap() {
            assert!(
                !matches!(
                    event.payload,
                    crate::EventPayload::Extension { event_id: 1, .. }
                ),
                "data/width/selection projection must not emit a user edit"
            );
        }
    }
}
