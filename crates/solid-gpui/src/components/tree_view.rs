//! A retained native tree with stable node identity and explicit lazy-branch requests.
use super::data_list::KeySelection;
use crate::native::{
    ControlledBinding, Event, EventDefinition, NativeChildren, NativeView, ViewCommand,
};
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Window, px,
};
use gpui_component::{
    Icon, IconName,
    tree::{TreeEvent, TreeItem, TreeState},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub expanded: Option<bool>,
    #[serde(default)]
    pub has_children: bool,
    #[serde(default = "loaded")]
    pub children_loaded: bool,
    #[serde(default)]
    pub children: Vec<TreeNode>,
}
fn loaded() -> bool {
    true
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct TreeProps {
    pub nodes: Vec<TreeNode>,
    pub selected_key: Option<String>,
    pub ack_edit_seq: u32,
    pub data_revision: u32,
    pub row_height: f32,
    pub indent: f32,
}
impl Default for TreeProps {
    fn default() -> Self {
        Self {
            nodes: vec![],
            selected_key: None,
            ack_edit_seq: 0,
            data_revision: 0,
            row_height: 32.,
            indent: 18.,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TreeExpansion {
    pub key: String,
    pub expanded: bool,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TreeLoadRequest {
    pub key: String,
    pub request_id: u32,
    pub data_revision: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TreeLoadCompletion {
    pub key: String,
    pub request_id: u32,
}
#[derive(Clone)]
struct Decoration {
    icon: Option<String>,
    expanded: Option<bool>,
    unloaded: bool,
}

pub struct Tree {
    state: Entity<TreeState>,
    props: TreeProps,
    items: HashMap<String, TreeItem>,
    decorations: Arc<HashMap<String, Decoration>>,
    event: Event<KeySelection>,
    children: NativeChildren,
    edit_seq: u32,
    request_seq: u32,
    pending: HashMap<String, u32>,
    _subscription: Subscription,
}
impl Tree {
    fn reconcile_nodes(
        nodes: &[TreeNode],
        old: &mut HashMap<String, TreeItem>,
        old_meta: &HashMap<String, Decoration>,
        items: &mut HashMap<String, TreeItem>,
        decorations: &mut HashMap<String, Decoration>,
    ) -> Vec<TreeItem> {
        nodes
            .iter()
            .map(|node| {
                let existed = old.contains_key(&node.key);
                let mut item = old
                    .remove(&node.key)
                    .unwrap_or_else(|| TreeItem::new(node.key.clone(), node.label.clone()));
                item.label = node.label.clone().into();
                item = item.has_children(node.has_children).disabled(node.disabled);
                if let Some(expanded) = node.expanded
                    && (!existed
                        || old_meta
                            .get(&node.key)
                            .is_none_or(|m| m.expanded != node.expanded))
                {
                    item = item.expanded(expanded);
                }
                item.children =
                    Self::reconcile_nodes(&node.children, old, old_meta, items, decorations);
                items.insert(node.key.clone(), item.clone());
                decorations.insert(
                    node.key.clone(),
                    Decoration {
                        icon: node.icon.clone(),
                        expanded: node.expanded,
                        unloaded: node.has_children && !node.children_loaded,
                    },
                );
                item
            })
            .collect()
    }
    fn request_children(&mut self, key: &str) {
        if !self.decorations.get(key).is_some_and(|m| m.unloaded)
            || self.pending.contains_key(key)
            || !self
                .event
                .related::<TreeLoadRequest>("loadChildren")
                .is_subscribed()
        {
            return;
        }
        self.request_seq = self
            .request_seq
            .checked_add(1)
            .expect("tree request sequence exhausted");
        self.pending.insert(key.into(), self.request_seq);
        self.event.related("loadChildren").emit(TreeLoadRequest {
            key: key.into(),
            request_id: self.request_seq,
            data_revision: self.props.data_revision,
        });
    }
    fn request_expanded(&mut self) {
        let keys: Vec<_> = self
            .items
            .iter()
            .filter(|(_, item)| item.is_expanded())
            .map(|(key, _)| key.clone())
            .collect();
        for key in keys {
            self.request_children(&key);
        }
    }
    fn on_event(
        &mut self,
        _: &Entity<TreeState>,
        event: &TreeEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            TreeEvent::Selected(key) => {
                self.edit_seq = self
                    .edit_seq
                    .checked_add(1)
                    .expect("tree selection sequence exhausted");
                self.event.emit(KeySelection {
                    value: key.as_ref().map(ToString::to_string),
                    edit_seq: self.edit_seq,
                    data_revision: self.props.data_revision,
                });
            }
            TreeEvent::Expanded(key) | TreeEvent::Collapsed(key) => {
                let expanded = matches!(event, TreeEvent::Expanded(_));
                self.event.related("expand").emit(TreeExpansion {
                    key: key.to_string(),
                    expanded,
                });
                if expanded {
                    self.request_children(key);
                }
            }
            TreeEvent::RightClicked(key) => self.event.related("rightClick").emit(key.to_string()),
            TreeEvent::Confirmed(key) => self.event.related("confirm").emit(key.to_string()),
        }
        cx.notify();
    }
}
#[crate::component]
impl NativeView for Tree {
    type Props = TreeProps;
    type Event = KeySelection;
    fn event_name() -> &'static str {
        "select"
    }
    fn slots() -> &'static [&'static str] {
        &["empty"]
    }
    fn controlled() -> Option<ControlledBinding> {
        Some(ControlledBinding {
            value_prop: "selectedKey",
            event_id: 1,
            sequence_field: "editSeq",
            ack_prop: "ackEditSeq",
        })
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![
            EventDefinition::new::<TreeExpansion>("expand"),
            EventDefinition::new::<TreeLoadRequest>("loadChildren"),
            EventDefinition::new::<String>("confirm"),
            EventDefinition::new::<String>("rightClick"),
        ]
    }
    fn validate_props(p: &TreeProps) -> Result<(), String> {
        if !p.row_height.is_finite()
            || !(16. ..=256.).contains(&p.row_height)
            || !p.indent.is_finite()
            || !(0. ..=64.).contains(&p.indent)
        {
            return Err("tree requires rowHeight 16..256 and indent 0..64".into());
        }
        fn check<'a>(
            nodes: &'a [TreeNode],
            depth: usize,
            keys: &mut HashSet<&'a str>,
        ) -> Result<(), String> {
            if depth > 48 {
                return Err("tree depth may not exceed 48".into());
            }
            for node in nodes {
                if node.key.is_empty() || !keys.insert(&node.key) || keys.len() > 100000 {
                    return Err(
                        "tree requires at most 100000 nonempty, globally unique keys".into(),
                    );
                }
                if !node.has_children && !node.children_loaded {
                    return Err("an unloaded node requires hasChildren".into());
                }
                check(&node.children, depth + 1, keys)?;
            }
            Ok(())
        }
        let mut keys = HashSet::new();
        check(&p.nodes, 0, &mut keys)?;
        if p.selected_key
            .as_ref()
            .is_some_and(|k| !keys.contains(k.as_str()))
        {
            return Err("tree selectedKey must exist in nodes".into());
        }
        Ok(())
    }
    fn mount(
        props: TreeProps,
        event: Event<KeySelection>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut items = HashMap::new();
        let mut decorations = HashMap::new();
        let roots = Self::reconcile_nodes(
            &props.nodes,
            &mut HashMap::new(),
            &HashMap::new(),
            &mut items,
            &mut decorations,
        );
        let state = cx.new(|cx| {
            let mut state = TreeState::new(cx).items(roots);
            if let Some(item) = props.selected_key.as_ref().and_then(|k| items.get(k)) {
                state.set_selected_item(Some(item), cx);
            }
            state
        });
        let subscription = cx.subscribe_in(&state, window, Self::on_event);
        let mut this = Self {
            state,
            props,
            items,
            decorations: Arc::new(decorations),
            event,
            children,
            edit_seq: 0,
            request_seq: 0,
            pending: HashMap::new(),
            _subscription: subscription,
        };
        this.request_expanded();
        this
    }
    fn update(&mut self, props: TreeProps, _: &mut Window, cx: &mut Context<Self>) {
        let data_changed = props.nodes != self.props.nodes;
        if data_changed {
            let mut items = HashMap::new();
            let mut decorations = HashMap::new();
            let roots = Self::reconcile_nodes(
                &props.nodes,
                &mut self.items,
                &self.decorations,
                &mut items,
                &mut decorations,
            );
            self.items = items;
            self.decorations = Arc::new(decorations);
            self.pending.retain(|key, _| self.items.contains_key(key));
            self.state
                .update(cx, |state, cx| state.set_items(roots, cx));
        }
        if (props.selected_key != self.props.selected_key
            || props.ack_edit_seq != self.props.ack_edit_seq)
            && (self.props.selected_key.is_none() && props.selected_key.is_some()
                || props.ack_edit_seq >= self.edit_seq)
        {
            let item = props.selected_key.as_ref().and_then(|k| self.items.get(k));
            self.state
                .update(cx, |state, cx| state.set_selected_item(item, cx));
        }
        self.props = props;
        if data_changed {
            self.request_expanded();
        }
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("focus", |this, (): (), window, cx| {
                this.state.update(cx, |state, cx| state.focus(window, cx));
                Ok(())
            }),
            ViewCommand::new("reveal", |this, key: String, _, cx| {
                if !this.items.contains_key(&key) {
                    return Err("tree key does not exist".into());
                }
                this.state.update(cx, |state, cx| {
                    state.reveal_item(&key.into(), gpui::ScrollStrategy::Center, cx)
                });
                Ok(())
            }),
            ViewCommand::new("setExpanded", |this, change: TreeExpansion, _, cx| {
                if this.state.update(cx, |state, cx| {
                    state.set_expanded(&change.key, change.expanded, cx)
                }) {
                    Ok(())
                } else {
                    Err("tree key must identify an enabled branch".into())
                }
            }),
            ViewCommand::new(
                "finishChildren",
                |this, completion: TreeLoadCompletion, _, _| {
                    if this.pending.get(&completion.key) != Some(&completion.request_id) {
                        return Err(
                            "tree response does not match the outstanding branch request".into(),
                        );
                    }
                    this.pending.remove(&completion.key);
                    Ok(())
                },
            ),
            ViewCommand::new("getExpanded", |this, (): (), _, _| {
                let mut keys: Vec<_> = this
                    .items
                    .iter()
                    .filter(|(_, item)| item.is_expanded())
                    .map(|(key, _)| key.clone())
                    .collect();
                keys.sort();
                Ok(keys)
            }),
        ]
    }
}
impl Render for Tree {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        if self.items.is_empty() {
            return self.children.slot("empty").into_any_element();
        }
        let decorations = self.decorations.clone();
        let height = self.props.row_height;
        let indent = self.props.indent;
        gpui_component::tree::tree(&self.state, move |_, entry, _, _, _| {
            let item = entry.item();
            let icon = decorations
                .get(item.id.as_ref())
                .and_then(|m| m.icon.as_ref())
                .map(|path| Icon::default().path(path.clone()))
                .unwrap_or_else(|| {
                    Icon::new(if entry.is_folder() {
                        if entry.is_expanded() {
                            IconName::FolderOpen
                        } else {
                            IconName::Folder
                        }
                    } else {
                        IconName::File
                    })
                });
            let chevron = if entry.is_folder() {
                Icon::new(if entry.is_expanded() {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .size_4()
                .into_any_element()
            } else {
                gpui::div().size_4().into_any_element()
            };
            gpui_component::list::ListItem::new(SharedString::from(item.id.to_string()))
                .h(px(height))
                .overflow_hidden()
                .pl(px(entry.depth() as f32 * indent + 4.))
                .child(
                    gpui_component::h_flex()
                        .items_center()
                        .gap_2()
                        .child(chevron)
                        .child(icon.size_4())
                        .child(item.label.clone()),
                )
        })
        .into_any_element()
    }
}
pub(crate) fn definition() -> crate::native::ComponentDefinition {
    __native_component_Tree()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    fn node(key: &str) -> TreeNode {
        TreeNode {
            key: key.into(),
            label: key.into(),
            icon: None,
            disabled: false,
            expanded: None,
            has_children: false,
            children_loaded: true,
            children: vec![],
        }
    }
    #[gpui::test]
    fn tree_reconciliation_preserves_expansion_selection_and_lazy_requests(
        cx: &mut gpui::TestAppContext,
    ) {
        let mut branch = node("branch");
        branch.children = vec![node("child")];
        branch.expanded = Some(true);
        let mut lazy = node("lazy");
        lazy.has_children = true;
        lazy.children_loaded = false;
        let fixture = Fixture::<Tree>::new(
            TreeProps {
                nodes: vec![branch.clone(), lazy.clone()],
                selected_key: Some("child".into()),
                ..Default::default()
            },
            cx,
        );
        fixture.update(cx, |v, _, cx| {
            v.state.update(cx, |state, cx| {
                assert!(state.set_expanded("branch", false, cx));
                assert_eq!(state.selected_item().unwrap().id.as_ref(), "child");
                assert_eq!(state.selected_index(), None);
                assert!(state.set_expanded("lazy", true, cx));
                assert!(state.set_expanded("lazy", true, cx));
            });
        });
        cx.run_until_parked();
        fixture.update(cx, |v, window, cx| {
            let id = v.state.entity_id();
            assert_eq!(v.pending.len(), 1);
            let request = v.pending["lazy"];
            branch.label = "renamed".into();
            v.update(
                TreeProps {
                    nodes: vec![lazy.clone(), branch.clone()],
                    selected_key: Some("child".into()),
                    data_revision: 2,
                    ..Default::default()
                },
                window,
                cx,
            );
            assert_eq!(v.state.entity_id(), id);
            assert_eq!(v.pending["lazy"], request);
            assert!(
                !v.items["branch"].is_expanded(),
                "unchanged explicit prop must not reset local expansion on unrelated data updates"
            );
            assert_eq!(
                v.state.read(cx).selected_item().unwrap().id.as_ref(),
                "child"
            );
            lazy.children = vec![node("loaded")];
            lazy.children_loaded = true;
            v.update(
                TreeProps {
                    nodes: vec![lazy.clone(), branch.clone()],
                    selected_key: Some("child".into()),
                    data_revision: 3,
                    ..Default::default()
                },
                window,
                cx,
            );
            assert_eq!(
                v.pending["lazy"], request,
                "completion remains valid after publishing children"
            );
            assert!(v.items["lazy"].is_expanded());
            assert!(v.state.read(cx).index_of(&"loaded".into()).is_some());
        });
        let mut loads = 0;
        while let Some(event) = fixture.runtime.take_event().unwrap() {
            if let crate::EventPayload::Extension { event_id: 4, .. } = event.payload {
                loads += 1;
            }
        }
        assert_eq!(loads, 1);
    }
}
