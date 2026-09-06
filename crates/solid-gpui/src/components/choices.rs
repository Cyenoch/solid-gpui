//! Value-keyed searchable delegates. Filtering is owned work; popup rendering reads native data.
use super::ControlSize;
use crate::native::{
    ControlledBinding, Event, EventDefinition, NativeChildren, NativeView, ViewCommand,
};
use gpui::{
    AnyElement, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Subscription, Task, Window, px,
};
use gpui_component::{
    IndexPath, Sizable,
    searchable_list::{SearchableListDelegate, SearchableListItem},
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}
impl SearchableListItem for Choice {
    type Value = String;
    fn title(&self) -> SharedString {
        self.label.clone().into()
    }
    fn value(&self) -> &String {
        &self.key
    }
    fn disabled(&self) -> bool {
        self.disabled
    }
    fn render(&self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut row = gpui_component::h_flex().gap_2().overflow_hidden();
        if let Some(path) = &self.icon {
            row = row.child(gpui_component::Icon::default().path(path.clone()));
        }
        let mut text = gpui_component::v_flex().overflow_hidden().child(
            gpui::div()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(self.label.clone()),
        );
        if let Some(description) = &self.description {
            text = text.child(
                gpui::div()
                    .text_xs()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(description.clone()),
            );
        }
        row.child(text)
    }
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
pub struct ChoiceGroup {
    pub key: String,
    #[serde(default)]
    pub label: Option<String>,
    pub items: Vec<Choice>,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceQuery {
    pub query: String,
    pub request_id: u32,
    pub data_revision: u32,
}
struct ChoiceData {
    groups: Vec<ChoiceGroup>,
    positions: HashMap<String, (usize, usize)>,
    search: Vec<Vec<String>>,
}
impl ChoiceData {
    fn new(groups: Vec<ChoiceGroup>) -> Self {
        let positions = groups
            .iter()
            .enumerate()
            .flat_map(|(s, g)| {
                g.items
                    .iter()
                    .enumerate()
                    .map(move |(r, i)| (i.key.clone(), (s, r)))
            })
            .collect();
        let search = groups
            .iter()
            .map(|g| {
                g.items
                    .iter()
                    .map(|i| {
                        format!(
                            "{}\n{}\n{}",
                            g.label.as_deref().unwrap_or_default(),
                            i.label,
                            i.keywords.join("\n")
                        )
                        .to_lowercase()
                    })
                    .collect()
            })
            .collect();
        Self {
            groups,
            positions,
            search,
        }
    }
    fn filter(&self, query: &str, filterable: bool) -> Vec<Vec<usize>> {
        let query = query.trim().to_lowercase();
        self.search
            .iter()
            .map(|items| {
                items
                    .iter()
                    .enumerate()
                    .filter_map(|(i, text)| {
                        (!filterable || query.is_empty() || text.contains(&query)).then_some(i)
                    })
                    .collect()
            })
            .collect()
    }
}
struct Choices {
    data: Arc<ChoiceData>,
    visible: Rc<RefCell<Vec<Vec<usize>>>>,
    filterable: bool,
    query_event: Event<ChoiceQuery>,
    request_id: Rc<Cell<u32>>,
    revision: u32,
}
impl Choices {
    fn new(
        data: Arc<ChoiceData>,
        query: &str,
        filterable: bool,
        query_event: Event<ChoiceQuery>,
        request_id: Rc<Cell<u32>>,
        revision: u32,
    ) -> Self {
        let visible = Rc::new(RefCell::new(data.filter(query, filterable)));
        Self {
            data,
            visible,
            filterable,
            query_event,
            request_id,
            revision,
        }
    }
    fn selection(&self, values: &[String]) -> Vec<(IndexPath, Choice)> {
        values
            .iter()
            .filter_map(|key| {
                let (s, r) = *self.data.positions.get(key)?;
                let item = self.data.groups[s].items[r].clone();
                let ix = self.position(key).unwrap_or(IndexPath {
                    section: usize::MAX,
                    row: r,
                    column: s,
                });
                Some((ix, item))
            })
            .collect()
    }
}
impl SearchableListDelegate for Choices {
    type Item = Choice;
    fn sections_count(&self, _: &App) -> usize {
        self.data.groups.len()
    }
    fn items_count(&self, section: usize) -> usize {
        self.visible.borrow().get(section).map_or(0, Vec::len)
    }
    fn item(&self, ix: IndexPath) -> Option<&Choice> {
        let row = *self.visible.borrow().get(ix.section)?.get(ix.row)?;
        self.data.groups.get(ix.section)?.items.get(row)
    }
    fn position<V>(&self, value: &V) -> Option<IndexPath>
    where
        Self::Item: SearchableListItem<Value = V>,
        V: PartialEq,
    {
        self.visible
            .borrow()
            .iter()
            .enumerate()
            .find_map(|(s, rows)| {
                rows.iter()
                    .position(|r| self.data.groups[s].items[*r].value() == value)
                    .map(|r| IndexPath::default().section(s).row(r))
            })
    }
    fn render_section_header(
        &self,
        section: usize,
        _: &mut Window,
        _: &mut App,
    ) -> Option<AnyElement> {
        if self.items_count(section) == 0 {
            return None;
        }
        self.data.groups.get(section)?.label.as_ref().map(|label| {
            gpui::div()
                .px_2()
                .py_1()
                .text_sm()
                .child(label.clone())
                .into_any_element()
        })
    }
    fn perform_search(&mut self, query: &str, _: &mut Window, cx: &mut App) -> Task<()> {
        let id = self
            .request_id
            .get()
            .checked_add(1)
            .expect("choice query sequence exhausted");
        self.request_id.set(id);
        self.query_event.emit(ChoiceQuery {
            query: query.into(),
            request_id: id,
            data_revision: self.revision,
        });
        let query = query.to_string();
        let data = self.data.clone();
        let filterable = self.filterable;
        let visible = self.visible.clone();
        let filtered = cx
            .background_executor()
            .spawn(async move { data.filter(&query, filterable) });
        cx.spawn(async move |_| {
            *visible.borrow_mut() = filtered.await;
        })
    }
}
macro_rules! choice_props {
    ($name:ident{$($field:ident:$ty:ty=$default:expr),*$(,)?})=>{
        #[crate::native_type]
        #[derive(Clone,Debug)]
        #[serde(default,rename_all="camelCase")]
        pub struct $name{pub items:Vec<ChoiceGroup>,pub data_revision:u32,pub ack_edit_seq:u32,pub disabled:bool,pub searchable:bool,pub filterable:bool,pub placeholder:String,pub search_placeholder:Option<String>,pub cleanable:bool,pub appearance:bool,pub size:ControlSize,pub menu_width:Option<f32>,pub menu_max_height:f32,$(pub $field:$ty,)*}
        impl Default for $name{fn default()->Self{Self{items:Vec::new(),data_revision:0,ack_edit_seq:0,disabled:false,searchable:false,filterable:true,placeholder:String::new(),search_placeholder:None,cleanable:false,appearance:true,size:ControlSize::Medium,menu_width:None,menu_max_height:320.,$($field:$default,)*}}}
    }
}
choice_props!(SelectProps{value:Option<String>=None,default_value:Option<String>=None,accessibility_label:Option<String>=None});
choice_props!(ComboboxProps{values:Option<Vec<String>>=None,default_values:Vec<String>=Vec::new(),multiple:bool=false});
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SelectChange {
    pub value: Option<String>,
    pub edit_seq: u32,
    pub data_revision: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComboboxChange {
    pub values: Vec<String>,
    pub edit_seq: u32,
    pub data_revision: u32,
}
fn validate(
    groups: &[ChoiceGroup],
    values: &[String],
    width: Option<f32>,
    height: f32,
) -> Result<(), String> {
    let mut keys = HashSet::new();
    let mut group_keys = HashSet::new();
    if groups.len() > 1000 || groups.iter().map(|g| g.items.len()).sum::<usize>() > 100000 {
        return Err("choice model exceeds 1000 groups or 100000 items".into());
    }
    for group in groups {
        if group.key.is_empty() || !group_keys.insert(&group.key) {
            return Err("choice group keys must be unique and nonempty".into());
        }
        for item in &group.items {
            if item.key.is_empty() || !keys.insert(&item.key) {
                return Err("choice item keys must be globally unique and nonempty".into());
            }
        }
    }
    if values.iter().collect::<HashSet<_>>().len() != values.len()
        || values.iter().any(|k| !keys.contains(k))
    {
        return Err("selected choice keys must be unique and present in items".into());
    }
    if !height.is_finite()
        || !(24. ..=10000.).contains(&height)
        || width.is_some_and(|v| !v.is_finite() || !(24. ..=10000.).contains(&v))
    {
        return Err("choice popup dimensions must be 24..10000 pixels".into());
    }
    Ok(())
}
fn binding(prop: &'static str) -> Option<ControlledBinding> {
    Some(ControlledBinding {
        value_prop: prop,
        event_id: 1,
        sequence_field: "editSeq",
        ack_prop: "ackEditSeq",
    })
}
pub struct Select {
    state: Entity<gpui_component::select::SelectState<Choices>>,
    props: SelectProps,
    data: Arc<ChoiceData>,
    events: Event<SelectChange>,
    request_id: Rc<Cell<u32>>,
    edit_seq: u32,
    children: NativeChildren,
    _subscription: Subscription,
}
#[crate::component]
impl NativeView for Select {
    type Props = SelectProps;
    type Event = SelectChange;
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        validate(
            &p.items,
            &p.value
                .as_ref()
                .or(p.default_value.as_ref())
                .cloned()
                .into_iter()
                .collect::<Vec<_>>(),
            p.menu_width,
            p.menu_max_height,
        )
    }
    fn slots() -> &'static [&'static str] {
        &["empty"]
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn controlled() -> Option<ControlledBinding> {
        binding("value")
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![EventDefinition::new::<ChoiceQuery>("query")]
    }
    fn mount(
        props: Self::Props,
        events: Event<Self::Event>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let data = Arc::new(ChoiceData::new(props.items.clone()));
        let request_id = Rc::new(Cell::new(0));
        let model = Choices::new(
            data.clone(),
            "",
            props.filterable,
            events.related("query"),
            request_id.clone(),
            props.data_revision,
        );
        let values: Vec<_> = props
            .value
            .as_ref()
            .or(props.default_value.as_ref())
            .cloned()
            .into_iter()
            .collect();
        let selection = model.selection(&values);
        let cursor = values.first().and_then(|v| model.position(v));
        let state = cx.new(|cx| {
            let mut state = gpui_component::select::SelectState::new(model, None, window, cx)
                .searchable(props.searchable);
            let model = Choices::new(
                data.clone(),
                "",
                props.filterable,
                events.related("query"),
                request_id.clone(),
                props.data_revision,
            );
            state.replace_model(model, selection, cursor, window, cx);
            state
        });
        let subscription = cx.subscribe(&state, |this, _, event, _| {
            let gpui_component::select::SelectEvent::Confirm(value) = event;
            this.edit_seq = this
                .edit_seq
                .checked_add(1)
                .expect("select edit sequence exhausted");
            this.events.emit(SelectChange {
                value: value.clone(),
                edit_seq: this.edit_seq,
                data_revision: this.props.data_revision,
            });
        });
        Self {
            state,
            props,
            data,
            events,
            request_id,
            edit_seq: 0,
            children,
            _subscription: subscription,
        }
    }
    fn update(&mut self, p: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        let model_changed = p.items != self.props.items
            || p.filterable != self.props.filterable
            || p.data_revision != self.props.data_revision;
        let value_changed = (p.value != self.props.value
            || p.ack_edit_seq != self.props.ack_edit_seq)
            && (self.props.value.is_none() && p.value.is_some() || p.ack_edit_seq >= self.edit_seq);
        if p.items != self.props.items {
            self.data = Arc::new(ChoiceData::new(p.items.clone()));
        }
        self.state.update(cx, |s, cx| {
            if model_changed || value_changed {
                let values: Vec<_> = if value_changed {
                    p.value.clone().into_iter().collect()
                } else {
                    s.selected_value().cloned().into_iter().collect()
                };
                let model = Choices::new(
                    self.data.clone(),
                    &s.query(cx),
                    p.filterable,
                    self.events.related("query"),
                    self.request_id.clone(),
                    p.data_revision,
                );
                let selection = model.selection(&values);
                let cursor = s
                    .cursor_value(cx)
                    .and_then(|v| model.position(&v))
                    .or_else(|| values.first().and_then(|v| model.position(v)));
                s.replace_model(model, selection, cursor, window, cx);
            }
            if p.searchable != self.props.searchable {
                s.set_searchable(p.searchable, cx);
            }
            if p.disabled && !self.props.disabled {
                s.set_open(false, cx);
            }
        });
        self.props = p;
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("focus", |this, (): (), window, cx| {
                if this.props.disabled {
                    return Err("disabled select cannot be focused".into());
                }
                this.state.update(cx, |s, cx| s.focus(window, cx));
                Ok(())
            }),
            ViewCommand::new("setOpen", |this, open: bool, _, cx| {
                if open && this.props.disabled {
                    return Err("disabled select cannot be opened".into());
                }
                this.state.update(cx, |s, cx| s.set_open(open, cx));
                Ok(())
            }),
            ViewCommand::new("setQuery", |this, query: String, window, cx| {
                this.state
                    .update(cx, |s, cx| s.set_query(query, window, cx));
                Ok(())
            }),
        ]
    }
}
impl Render for Select {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut v = gpui_component::select::Select::new(&self.state)
            .placeholder(self.props.placeholder.clone())
            .cleanable(self.props.cleanable)
            .disabled(self.props.disabled)
            .appearance(self.props.appearance)
            .with_size(self.props.size)
            .menu_max_h(px(self.props.menu_max_height));
        if let Some(width) = self.props.menu_width {
            v = v.menu_width(px(width));
        }
        if let Some(label) = &self.props.accessibility_label {
            v = v.accessibility_label(label.clone());
        }
        if let Some(text) = &self.props.search_placeholder {
            v = v.search_placeholder(text.clone());
        }
        let empty = self.children.slot("empty");
        if !empty.is_empty() {
            v = v.empty(move |_, _| empty.clone());
        }
        v
    }
}
pub struct Combobox {
    state: Entity<gpui_component::combobox::ComboboxState<Choices>>,
    props: ComboboxProps,
    data: Arc<ChoiceData>,
    events: Event<ComboboxChange>,
    request_id: Rc<Cell<u32>>,
    edit_seq: u32,
    children: NativeChildren,
    _subscription: Subscription,
}
#[crate::component]
impl NativeView for Combobox {
    type Props = ComboboxProps;
    type Event = ComboboxChange;
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        let values = p.values.as_ref().unwrap_or(&p.default_values);
        validate(&p.items, values, p.menu_width, p.menu_max_height)?;
        if !p.multiple && values.len() > 1 {
            return Err("single combobox accepts at most one selected key".into());
        }
        Ok(())
    }
    fn slots() -> &'static [&'static str] {
        &["empty", "trigger", "footer"]
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn controlled() -> Option<ControlledBinding> {
        binding("values")
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![
            EventDefinition::new::<ChoiceQuery>("query"),
            EventDefinition::new::<Vec<String>>("confirm"),
        ]
    }
    fn mount(
        props: Self::Props,
        events: Event<Self::Event>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let data = Arc::new(ChoiceData::new(props.items.clone()));
        let request_id = Rc::new(Cell::new(0));
        let model = Choices::new(
            data.clone(),
            "",
            props.filterable,
            events.related("query"),
            request_id.clone(),
            props.data_revision,
        );
        let values = props.values.as_ref().unwrap_or(&props.default_values);
        let selection = model.selection(values);
        let cursor = values.first().and_then(|v| model.position(v));
        let state = cx.new(|cx| {
            let mut s = gpui_component::combobox::ComboboxState::new(model, Vec::new(), window, cx)
                .multiple(props.multiple)
                .searchable(props.searchable);
            let model = Choices::new(
                data.clone(),
                "",
                props.filterable,
                events.related("query"),
                request_id.clone(),
                props.data_revision,
            );
            s.replace_model(model, selection, cursor, window, cx);
            s
        });
        let subscription = cx.subscribe(&state, |this, _, event, _| {
            use gpui_component::combobox::ComboboxEvent;
            match event {
                ComboboxEvent::Change(values) => {
                    this.edit_seq = this
                        .edit_seq
                        .checked_add(1)
                        .expect("combobox edit sequence exhausted");
                    this.events.emit(ComboboxChange {
                        values: values.clone(),
                        edit_seq: this.edit_seq,
                        data_revision: this.props.data_revision,
                    });
                }
                ComboboxEvent::Confirm(values) => {
                    this.events.related("confirm").emit(values.clone())
                }
            }
        });
        Self {
            state,
            props,
            data,
            events,
            request_id,
            edit_seq: 0,
            children,
            _subscription: subscription,
        }
    }
    fn update(&mut self, p: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        let model_changed = p.items != self.props.items
            || p.filterable != self.props.filterable
            || p.data_revision != self.props.data_revision;
        let entering = self.props.values.is_none() && p.values.is_some();
        let value_changed = p.values.is_some()
            && (entering || p.ack_edit_seq >= self.edit_seq)
            && (p.values != self.props.values || p.ack_edit_seq != self.props.ack_edit_seq);
        if p.items != self.props.items {
            self.data = Arc::new(ChoiceData::new(p.items.clone()));
        }
        self.state.update(cx, |s, cx| {
            if model_changed || value_changed {
                let mut values = if value_changed {
                    p.values.clone().expect("controlled values")
                } else {
                    s.selected_values()
                };
                if !p.multiple {
                    values.truncate(1);
                }
                let model = Choices::new(
                    self.data.clone(),
                    &s.query(cx),
                    p.filterable,
                    self.events.related("query"),
                    self.request_id.clone(),
                    p.data_revision,
                );
                let selection = model.selection(&values);
                let cursor = s
                    .cursor_value(cx)
                    .and_then(|v| model.position(&v))
                    .or_else(|| values.first().and_then(|v| model.position(v)));
                s.replace_model(model, selection, cursor, window, cx);
            }
            if p.multiple != self.props.multiple {
                s.set_multiple(p.multiple, cx);
            }
            if p.searchable != self.props.searchable {
                s.set_searchable(p.searchable, cx);
            }
            if p.disabled && !self.props.disabled {
                s.set_open(false, cx);
            }
        });
        self.props = p;
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("focus", |this, (): (), window, cx| {
                if this.props.disabled {
                    return Err("disabled combobox cannot be focused".into());
                }
                this.state.update(cx, |s, cx| s.focus(window, cx));
                Ok(())
            }),
            ViewCommand::new("setOpen", |this, open: bool, _, cx| {
                if open && this.props.disabled {
                    return Err("disabled combobox cannot be opened".into());
                }
                this.state.update(cx, |s, cx| s.set_open(open, cx));
                Ok(())
            }),
            ViewCommand::new("setQuery", |this, query: String, window, cx| {
                this.state
                    .update(cx, |s, cx| s.set_query(query, window, cx));
                Ok(())
            }),
        ]
    }
}
impl Render for Combobox {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut v = gpui_component::combobox::Combobox::new(&self.state)
            .placeholder(self.props.placeholder.clone())
            .cleanable(self.props.cleanable)
            .disabled(self.props.disabled)
            .appearance(self.props.appearance)
            .with_size(self.props.size)
            .menu_max_h(px(self.props.menu_max_height));
        if let Some(width) = self.props.menu_width {
            v = v.menu_width(px(width));
        }
        if let Some(text) = &self.props.search_placeholder {
            v = v.search_placeholder(text.clone());
        }
        let empty = self.children.slot("empty");
        if !empty.is_empty() {
            v = v.empty(move |_, _| empty.clone());
        }
        let trigger = self.children.slot("trigger");
        if !trigger.is_empty() {
            v = v.render_trigger(move |_, _, _| trigger.clone());
        }
        let footer = self.children.slot("footer");
        if !footer.is_empty() {
            v = v.footer(move |_, _| footer.clone());
        }
        v
    }
}
pub(crate) fn definitions() -> Vec<crate::native::ComponentDefinition> {
    vec![__native_component_Select(), __native_component_Combobox()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    fn groups(reverse: bool) -> Vec<ChoiceGroup> {
        let mut items = vec![
            Choice {
                key: "a".into(),
                label: "Alpha".into(),
                keywords: vec![],
                disabled: false,
                description: None,
                icon: None,
            },
            Choice {
                key: "b".into(),
                label: "Beta".into(),
                keywords: vec![],
                disabled: false,
                description: None,
                icon: None,
            },
        ];
        if reverse {
            items.reverse();
            items[1].label = "Alpha updated".into();
        }
        vec![ChoiceGroup {
            key: "g".into(),
            label: None,
            items,
        }]
    }
    #[gpui::test]
    fn replacing_filtered_choice_models_keeps_values_query_and_entities(
        cx: &mut gpui::TestAppContext,
    ) {
        let single = Fixture::<Select>::new(
            SelectProps {
                items: groups(false),
                value: Some("a".into()),
                searchable: true,
                ..Default::default()
            },
            cx,
        );
        let multi = Fixture::<Combobox>::new(
            ComboboxProps {
                items: groups(false),
                values: Some(vec!["a".into(), "b".into()]),
                multiple: true,
                searchable: true,
                ..Default::default()
            },
            cx,
        );
        single.update(cx, |v, window, cx| {
            let id = v.state.entity_id();
            v.state.update(cx, |s, cx| s.set_query("Beta", window, cx));
            v.update(
                SelectProps {
                    items: groups(true),
                    value: Some("a".into()),
                    searchable: true,
                    data_revision: 1,
                    ..Default::default()
                },
                window,
                cx,
            );
            assert_eq!(v.state.entity_id(), id);
            assert_eq!(v.state.read(cx).selected_value(), Some(&String::from("a")));
            assert_eq!(v.state.read(cx).query(cx).as_ref(), "Beta");
        });
        multi.update(cx, |v, window, cx| {
            let id = v.state.entity_id();
            v.state.update(cx, |s, cx| s.set_query("Beta", window, cx));
            v.update(
                ComboboxProps {
                    items: groups(true),
                    values: Some(vec!["a".into(), "b".into()]),
                    multiple: true,
                    searchable: true,
                    data_revision: 1,
                    ..Default::default()
                },
                window,
                cx,
            );
            assert_eq!(v.state.entity_id(), id);
            assert_eq!(v.state.read(cx).selected_values(), vec!["a", "b"]);
            assert_eq!(v.state.read(cx).query(cx).as_ref(), "Beta");
        });
        cx.run_until_parked();
        single.update(cx, |v, _, cx| {
            assert_eq!(v.state.read(cx).selected_value(), Some(&String::from("a")));
            assert_eq!(v.state.read(cx).cursor_value(cx), Some("b".to_string()));
        });
        multi.update(cx, |v, _, cx| {
            assert_eq!(v.state.read(cx).selected_values(), vec!["a", "b"])
        });
    }
}
