//! OS menus carry revocable view-scoped action routes; the OS owns each open menu snapshot.
use super::{menus::MenuSelection, plot::coordinate};
use crate::native::{ComponentDefinition, Event, NativeChildren, NativeView, ViewCommand};
use gpui::{
    Action, App, Context, Global, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Render, WeakEntity, Window, div, point, px,
};
use gpui_component::native_menu::NativeMenu as OsMenu;
use std::collections::{HashMap, HashSet};
#[derive(Action, Clone, PartialEq, serde::Deserialize)]
#[action(namespace=solid_gpui,no_json)]
struct NativePopupAction {
    owner: u64,
    generation: u64,
    item: String,
}
#[derive(Default)]
struct NativeMenuRoutes(HashMap<u64, WeakEntity<NativeMenu>>);
impl Global for NativeMenuRoutes {}
fn init(cx: &mut App) {
    if cx.try_global::<NativeMenuRoutes>().is_some() {
        return;
    }
    cx.set_global(NativeMenuRoutes::default());
    cx.on_action(|a: &NativePopupAction, cx| {
        if let Some(view) = cx
            .global::<NativeMenuRoutes>()
            .0
            .get(&a.owner)
            .and_then(WeakEntity::upgrade)
        {
            view.update(cx, |menu, _| {
                // An OS popup is asynchronous and retains its original actions.
                // Replacing props revokes that snapshot, even if an ID is reused.
                if a.generation == menu.generation
                    && menu.props.enabled
                    && selectable(&menu.props.items, &a.item)
                {
                    menu.event.emit(MenuSelection { id: a.item.clone() });
                }
            });
        }
    });
}
fn yes() -> bool {
    true
}
#[crate::native_type]
#[derive(Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NativeMenuEntry {
    Separator,
    Item {
        id: String,
        label: String,
        #[serde(default)]
        disabled: bool,
        #[serde(default)]
        checked: bool,
        #[serde(default)]
        icon: Option<String>,
    },
    Label {
        label: String,
    },
    Submenu {
        label: String,
        items: Vec<NativeMenuEntry>,
        #[serde(default)]
        disabled: bool,
    },
}
#[crate::native_type]
#[derive(Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
pub enum NativeMenuTrigger {
    #[default]
    ContextMenu,
    Press,
    Manual,
}
#[crate::native_type]
#[derive(Clone)]
pub struct NativeMenuProps {
    pub items: Vec<NativeMenuEntry>,
    #[serde(default)]
    pub trigger: NativeMenuTrigger,
    #[serde(default = "yes")]
    pub enabled: bool,
}
#[crate::native_type]
pub struct NativeMenuPosition {
    pub x: f32,
    pub y: f32,
}
fn validate(items: &[NativeMenuEntry]) -> Result<(), String> {
    fn visit(
        items: &[NativeMenuEntry],
        depth: usize,
        count: &mut usize,
        ids: &mut HashSet<String>,
    ) -> Result<(), String> {
        *count += items.len();
        if depth > 16 || *count > 1024 {
            return Err("native menus support at most 1024 entries and 16 submenu levels".into());
        }
        for item in items {
            match item {
                NativeMenuEntry::Item { id, .. } if id.is_empty() || !ids.insert(id.clone()) => {
                    return Err("native menu action ids must be non-empty and unique".into());
                }
                NativeMenuEntry::Submenu { items, .. } => visit(items, depth + 1, count, ids)?,
                _ => {}
            }
        }
        Ok(())
    }
    visit(items, 0, &mut 0, &mut HashSet::new())
}
fn selectable(items: &[NativeMenuEntry], id: &str) -> bool {
    items.iter().any(|item| match item {
        NativeMenuEntry::Item {
            id: item_id,
            disabled,
            ..
        } => item_id == id && !disabled,
        NativeMenuEntry::Submenu {
            items, disabled, ..
        } => !disabled && selectable(items, id),
        _ => false,
    })
}

fn build(items: &[NativeMenuEntry], owner: u64, generation: u64) -> OsMenu {
    items.iter().fold(OsMenu::new(), |m, item| match item {
        NativeMenuEntry::Separator => m.separator(),
        NativeMenuEntry::Label { label } => m.menu_with(label.clone(), true, false, None, None),
        NativeMenuEntry::Item {
            id,
            label,
            disabled,
            checked,
            icon,
        } => m.menu_with(
            label.clone(),
            *disabled,
            *checked,
            icon.as_ref()
                .map(|v| gpui_component::Icon::default().path(v.clone())),
            Some(Box::new(NativePopupAction {
                owner,
                generation,
                item: id.clone(),
            })),
        ),
        NativeMenuEntry::Submenu {
            label,
            items,
            disabled,
        } => m.submenu_with_disabled(label.clone(), build(items, owner, generation), *disabled),
    })
}
struct NativeMenu {
    props: NativeMenuProps,
    children: NativeChildren,
    owner: u64,
    generation: u64,
    event: Event<MenuSelection>,
}
impl NativeMenu {
    fn show(&self, p: NativeMenuPosition, window: &mut Window, cx: &mut App) -> Result<(), String> {
        coordinate(p.x)?;
        coordinate(p.y)?;
        if !self.props.enabled {
            return Err("native menu is disabled".into());
        }
        build(&self.props.items, self.owner, self.generation).show(
            point(px(p.x), px(p.y)),
            window,
            cx,
        );
        Ok(())
    }
}
impl NativeView for NativeMenu {
    type Props = NativeMenuProps;
    type Event = MenuSelection;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "select"
    }
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        validate(&p.items)
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        init(cx);
        let owner = cx.entity_id().as_u64();
        let view = cx.weak_entity();
        let routes = &mut cx.global_mut::<NativeMenuRoutes>().0;
        routes.retain(|_, v| v.upgrade().is_some());
        routes.insert(owner, view);
        Self {
            props,
            children,
            owner,
            generation: 0,
            event,
        }
    }
    fn update(&mut self, p: Self::Props, _: &mut Window, cx: &mut Context<Self>) {
        self.generation += 1;
        self.props = p;
        cx.notify();
    }
    fn unmount(&mut self, _: &mut Window, cx: &mut App) {
        cx.global_mut::<NativeMenuRoutes>().0.remove(&self.owner);
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![ViewCommand::new(
            "show",
            |v, p: NativeMenuPosition, w, cx| v.show(p, w, cx),
        )]
    }
}
impl Render for NativeMenu {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let el = div()
            .id("native-menu-trigger")
            .child(self.children.content());
        if !self.props.enabled {
            return el;
        }
        let button = match self.props.trigger {
            NativeMenuTrigger::ContextMenu => MouseButton::Right,
            NativeMenuTrigger::Press => MouseButton::Left,
            NativeMenuTrigger::Manual => return el,
        };
        el.on_mouse_down(
            button,
            cx.listener(|this, event: &gpui::MouseDownEvent, w, cx| {
                let _ = this.show(
                    NativeMenuPosition {
                        x: event.position.x.as_f32(),
                        y: event.position.y.as_f32(),
                    },
                    w,
                    cx,
                );
                cx.stop_propagation();
            }),
        )
    }
}
pub(super) fn definition() -> ComponentDefinition {
    ComponentDefinition::view::<NativeMenu>("NativeMenu")
        .with_contract(include_str!("native_menu.rs"))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventPayload, components::test_support::Fixture};
    #[gpui::test]
    fn os_menu_actions_require_a_live_enabled_snapshot_and_are_revoked_on_unmount(
        cx: &mut gpui::TestAppContext,
    ) {
        let props = NativeMenuProps {
            items: vec![NativeMenuEntry::Item {
                id: "copy".into(),
                label: "Copy".into(),
                disabled: false,
                checked: true,
                icon: None,
            }],
            trigger: NativeMenuTrigger::Manual,
            enabled: true,
        };
        let f = Fixture::<NativeMenu>::new(props, cx);
        cx.run_until_parked();
        let owner = f.update(cx, |v, _, _| {
            assert!(!build(&v.props.items, v.owner, v.generation).is_empty());
            v.owner
        });
        cx.update(|cx| {
            cx.dispatch_action(&NativePopupAction {
                owner,
                generation: 0,
                item: "copy".into(),
            })
        });
        cx.run_until_parked();
        let e = f.runtime.take_event().unwrap().unwrap();
        let EventPayload::Extension { fields, .. } = e.payload else {
            panic!("native selection event")
        };
        let crate::protocol::ExtensionValue::Bytes(bytes) = &fields[0].value else {
            panic!("typed event")
        };
        assert_eq!(
            crate::native::decode_json::<MenuSelection>(bytes)
                .unwrap()
                .id,
            "copy"
        );
        f.update(cx, |v, w, cx| {
            let mut props = v.props.clone();
            props.items = vec![NativeMenuEntry::Submenu {
                label: "Editing".into(),
                disabled: true,
                items: props.items,
            }];
            v.update(props, w, cx);
        });
        for (generation, item) in [(0, "copy"), (1, "copy"), (1, "missing")] {
            cx.update(|cx| {
                cx.dispatch_action(&NativePopupAction {
                    owner,
                    generation,
                    item: item.into(),
                })
            });
            cx.run_until_parked();
            assert!(f.runtime.take_event().unwrap().is_none());
        }
        f.update(cx, |v, w, cx| {
            let mut props = v.props.clone();
            let NativeMenuEntry::Submenu { disabled, .. } = &mut props.items[0] else {
                panic!("submenu fixture");
            };
            *disabled = false;
            v.update(props, w, cx);
        });
        cx.update(|cx| {
            cx.dispatch_action(&NativePopupAction {
                owner,
                generation: 1,
                item: "copy".into(),
            })
        });
        cx.run_until_parked();
        assert!(
            f.runtime.take_event().unwrap().is_none(),
            "re-enabling an ID must not revive an old popup snapshot"
        );
        cx.update(|cx| {
            cx.dispatch_action(&NativePopupAction {
                owner,
                generation: 2,
                item: "copy".into(),
            })
        });
        cx.run_until_parked();
        assert!(f.runtime.take_event().unwrap().is_some());
        f.update(cx, |v, w, cx| v.unmount(w, cx));
        cx.update(|cx| {
            cx.dispatch_action(&NativePopupAction {
                owner,
                generation: 2,
                item: "copy".into(),
            })
        });
        cx.run_until_parked();
        assert!(f.runtime.take_event().unwrap().is_none());
    }
}
