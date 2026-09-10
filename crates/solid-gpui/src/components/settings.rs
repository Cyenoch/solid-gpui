//! Native settings navigation with typed page/group/item/field descriptors.
use super::icon_source::ComponentIcon;
use super::{
    ControlSize,
    primitives::{GroupBoxVariant, Orientation},
    resizable::{PanelLimits, PanelSize},
};
use crate::{
    ExtensionChildSummary,
    native::{
        ComponentDefinition, ControlledBinding, ElementContext, Event, EventDefinition,
        NativeChildren, NativeView, ViewCommand,
    },
};
use gpui::{AppContext, Context, Entity, IntoElement, Render, SharedString, Window, px};
use gpui_component::{
    ComponentChild, Sizable,
    setting::{self, SettingField, SettingGroup, SettingItem, SettingPage},
};
use std::collections::HashSet;

#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingPageProps {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<ComponentIcon>,
    #[serde(default)]
    pub default_open: bool,
    #[serde(default = "yes")]
    pub resettable: bool,
}
fn yes() -> bool {
    true
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingGroupProps {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingItemProps {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub orientation: Orientation,
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct SettingFieldProps {
    pub dirty: bool,
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct SettingCustomItemProps {
    pub dirty: bool,
    pub disabled: bool,
    pub keywords: Vec<String>,
}
fn page(p: &SettingPageProps, cx: &mut ElementContext) -> SettingPage {
    let suffix = cx.slot("titleSuffix");
    let mut page = SettingPage::new(p.name.clone(), p.title.clone())
        .default_open(p.default_open)
        .resettable(p.resettable)
        .groups(cx.typed_children::<SettingGroup>());
    if !suffix.is_empty() {
        page = page.title_suffix(move |_, _| suffix.clone());
    }
    if let Some(v) = &p.description {
        page = page.description(v.clone());
    }
    if let Some(v) = &p.icon {
        page = page.icon(v.native());
    }
    page
}
fn group(p: &SettingGroupProps, cx: &mut ElementContext) -> SettingGroup {
    let mut group = SettingGroup::new(p.name.clone()).items(cx.typed_children::<SettingItem>());
    if let Some(v) = &p.title {
        group = group.title(v.clone());
    }
    if let Some(v) = &p.description {
        group = group.description(v.clone());
    }
    group
}
fn item(p: &SettingItemProps, cx: &mut ElementContext) -> SettingItem {
    let field: ComponentChild<SettingField<SharedString>> = cx
        .typed_children::<SettingField<SharedString>>()
        .into_iter()
        .next()
        .expect("one validated field")
        .into();
    let mut item = SettingItem::new(cx.id(), p.title.clone(), field)
        .disabled(p.disabled)
        .keywords(p.keywords.clone())
        .layout(p.orientation.into());
    if let Some(v) = &p.description {
        item = item.description(v.clone());
    }
    item
}
fn field(p: &SettingFieldProps, cx: &mut ElementContext) -> SettingField<SharedString> {
    let content = cx.content();
    let dirty = p.dirty;
    let reset = cx.event::<()>("reset");
    SettingField::render(move |_, _, _| content.clone())
        .on_reset(move |_| dirty, move |_, _| reset.emit(()))
}
fn custom_item(p: &SettingCustomItemProps, cx: &mut ElementContext) -> SettingItem {
    let content = cx.content();
    let dirty = p.dirty;
    let reset = cx.event::<()>("reset");
    SettingItem::render(cx.id(), move |_, _, _| content.clone())
        .disabled(p.disabled)
        .keywords(p.keywords.clone())
        .on_reset(move |_| dirty, move |_, _| reset.emit(()))
}
fn valid_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 256 {
        Err("setting names must contain 1 to 256 bytes".into())
    } else {
        Ok(())
    }
}
fn unique_names<'a>(names: impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut seen = HashSet::new();
    for name in names {
        valid_name(name)?;
        if !seen.insert(name) {
            return Err(format!("duplicate setting name: {name}"));
        }
    }
    Ok(())
}
#[crate::native_type]
#[derive(Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSelection {
    pub page: String,
    #[serde(default)]
    pub group: Option<String>,
}
impl From<&setting::SettingSelection> for SettingsSelection {
    fn from(v: &setting::SettingSelection) -> Self {
        Self {
            page: v.page.to_string(),
            group: v.group.as_ref().map(ToString::to_string),
        }
    }
}
impl From<SettingsSelection> for setting::SettingSelection {
    fn from(v: SettingsSelection) -> Self {
        Self {
            page: v.page.into(),
            group: v.group.map(Into::into),
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct SettingsProps {
    pub selection: Option<SettingsSelection>,
    pub ack_edit_seq: Option<u32>,
    pub default_query: String,
    pub size: ControlSize,
    pub group_variant: GroupBoxVariant,
    pub sidebar_width: PanelSize,
    pub sidebar_limits: PanelLimits,
}
impl Default for SettingsProps {
    fn default() -> Self {
        Self {
            selection: None,
            ack_edit_seq: None,
            default_query: String::new(),
            size: ControlSize::default(),
            group_variant: GroupBoxVariant::default(),
            sidebar_width: PanelSize(250.),
            sidebar_limits: PanelLimits {
                min: 160.,
                max: 360.,
            },
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingsChange {
    pub selection: SettingsSelection,
    pub edit_seq: u32,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSnapshot {
    pub selection: Option<SettingsSelection>,
    pub query: String,
}
struct Settings {
    props: SettingsProps,
    children: NativeChildren,
    state: Entity<setting::SettingsState>,
    seq: u32,
    _subscription: gpui::Subscription,
}
impl Settings {
    fn validate_selection(
        &self,
        selection: &SettingsSelection,
        cx: &gpui::App,
    ) -> Result<(), String> {
        let pages = self.children.typed_children::<SettingPage>(cx);
        let mut found = false;
        for page in pages {
            // NativeChild deliberately keeps its value behind the same render boundary.
            let page: ComponentChild<SettingPage> = page.into();
            if page.key().as_ref() == selection.page
                && selection.group.as_ref().is_none_or(|g| page.has_group(g))
            {
                found = true;
                break;
            }
        }
        if found {
            Ok(())
        } else {
            Err("selection does not address a committed page/group".into())
        }
    }
}
impl NativeView for Settings {
    type Props = SettingsProps;
    type Event = SettingsChange;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "change"
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
        vec![EventDefinition::new::<String>("searchChange")]
    }
    fn validate_props(p: &SettingsProps) -> Result<(), String> {
        if p.sidebar_width.0 < p.sidebar_limits.min || p.sidebar_width.0 > p.sidebar_limits.max {
            return Err("sidebarWidth must be within sidebarLimits".into());
        }
        if let Some(selection) = &p.selection {
            valid_name(&selection.page)?;
            if let Some(group) = &selection.group {
                valid_name(group)?;
            }
        }
        Ok(())
    }
    fn validate_children(
        p: &SettingsProps,
        children: &ExtensionChildSummary,
    ) -> Result<(), String> {
        let pages: Vec<SettingPageProps> = children.native_props()?;
        unique_names(pages.iter().map(|p| p.name.as_str()))?;
        if let Some(selection) = &p.selection {
            let index = pages
                .iter()
                .position(|page| page.name == selection.page)
                .ok_or("selection.page must address a committed page")?;
            if let Some(group) = &selection.group {
                let groups: Vec<SettingGroupProps> = children
                    .child(index)
                    .ok_or("missing page composition")?
                    .native_props()?;
                if !groups.iter().any(|g| &g.name == group) {
                    return Err("selection.group must address a committed group".into());
                }
            }
        }
        Ok(())
    }
    fn mount(
        props: SettingsProps,
        event: Event<SettingsChange>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|cx| setting::SettingsState::new(window, cx));
        state.update(cx, |s, cx| {
            if let Some(v) = &props.selection {
                s.select(v.clone().into(), cx);
            }
            if !props.default_query.is_empty() {
                s.set_query(props.default_query.clone(), window, cx);
            }
        });
        let search = event.related::<String>("searchChange");
        let subscription = cx.subscribe(&state, move |this, _, e, cx| {
            match e {
                setting::SettingsEvent::SelectionChanged(selection) => {
                    this.seq = this
                        .seq
                        .checked_add(1)
                        .expect("settings sequence exhausted");
                    event.emit(SettingsChange {
                        selection: selection.into(),
                        edit_seq: this.seq,
                    });
                }
                setting::SettingsEvent::SearchChanged(query) => search.emit(query.to_string()),
            }
            cx.notify();
        });
        Self {
            props,
            children,
            state,
            seq: 0,
            _subscription: subscription,
        }
    }
    fn update(&mut self, props: SettingsProps, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            if props.ack_edit_seq.is_none_or(|ack| ack >= self.seq)
                && (props.selection != self.props.selection
                    || props.ack_edit_seq != self.props.ack_edit_seq)
                && let Some(v) = &props.selection
            {
                s.select(v.clone().into(), cx);
            }
        });
        self.props = props;
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("select", |this, selection: SettingsSelection, _, cx| {
                this.validate_selection(&selection, cx)?;
                this.state
                    .update(cx, |s, cx| s.select(selection.into(), cx));
                Ok(())
            }),
            ViewCommand::new("setQuery", |this, query: String, window, cx| {
                this.state
                    .update(cx, |s, cx| s.set_query(query, window, cx));
                Ok(())
            }),
            ViewCommand::new("getState", |this, (): (), _, cx| {
                let state = this.state.read(cx);
                Ok(SettingsSnapshot {
                    selection: state.selected().map(Into::into),
                    query: state.query(cx).to_string(),
                })
            }),
        ]
    }
}
impl Render for Settings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let variant = match self.props.group_variant {
            GroupBoxVariant::Normal => gpui_component::group_box::GroupBoxVariant::Normal,
            GroupBoxVariant::Fill => gpui_component::group_box::GroupBoxVariant::Fill,
            GroupBoxVariant::Outline => gpui_component::group_box::GroupBoxVariant::Outline,
        };
        setting::Settings::new("settings")
            .with_state(&self.state)
            .with_size(self.props.size)
            .with_group_variant(variant)
            .sidebar_width(px(self.props.sidebar_width.0))
            .sidebar_size_range(
                px(self.props.sidebar_limits.min)..px(self.props.sidebar_limits.max),
            )
            .pages(self.children.typed_children::<SettingPage>(cx))
    }
}
pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::view::<Settings>("Settings").with_child_type::<SettingPage>(),
        ComponentDefinition::descriptor::<SettingPageProps, _>("SettingPage", vec![], page)
            .with_slots(&["titleSuffix"])
            .with_child_type::<SettingGroup>()
            .with_validation::<SettingPageProps>(|p, c| {
                valid_name(&p.name)?;
                let groups: Vec<SettingGroupProps> = c.native_props()?;
                unique_names(groups.iter().map(|g| g.name.as_str()))
            }),
        ComponentDefinition::styled_descriptor::<SettingGroupProps, _>(
            "SettingGroup",
            vec![],
            group,
        )
        .with_child_type::<SettingItem>()
        .with_validation::<SettingGroupProps>(|p, _| valid_name(&p.name)),
        ComponentDefinition::descriptor::<SettingItemProps, _>("SettingItem", vec![], item)
            .with_child_type::<SettingField<SharedString>>()
            .with_validation::<SettingItemProps>(|_, c| {
                if c.content_count() == 1 {
                    Ok(())
                } else {
                    Err("SettingItem requires exactly one SettingField".into())
                }
            }),
        ComponentDefinition::styled_descriptor::<SettingFieldProps, _>(
            "SettingField",
            vec![EventDefinition::new::<()>("reset")],
            field,
        ),
        ComponentDefinition::descriptor::<SettingCustomItemProps, _>(
            "SettingCustomItem",
            vec![EventDefinition::new::<()>("reset")],
            custom_item,
        ),
    ]
    .into_iter()
    .map(|d| d.with_contract(include_str!("settings.rs")))
    .collect()
}
