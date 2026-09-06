use super::data_list::{DataQuery, KeySelection};
use crate::native::{
    ControlledBinding, Event, EventDefinition, NativeChildren, NativeView, ViewCommand,
};
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window,
    prelude::FluentBuilder, px,
};
use gpui_component::{
    Disableable,
    command::{CommandEntry, CommandGroup, CommandItem, CommandState},
};
use std::{cell::Cell, collections::HashSet, rc::Rc};
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandChoice {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub shortcut: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub checked: bool,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
pub struct CommandSection {
    pub key: String,
    #[serde(default)]
    pub label: Option<String>,
    pub items: Vec<CommandChoice>,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct CommandProps {
    pub groups: Vec<CommandSection>,
    pub query: Option<String>,
    pub selected_key: Option<String>,
    pub ack_edit_seq: u32,
    pub data_revision: u32,
    pub searchable: bool,
    pub filterable: bool,
    pub loading: bool,
    pub bordered: bool,
    pub max_height: f32,
    pub placeholder: Option<String>,
}
impl Default for CommandProps {
    fn default() -> Self {
        Self {
            groups: vec![],
            query: None,
            selected_key: None,
            ack_edit_seq: 0,
            data_revision: 0,
            searchable: true,
            filterable: true,
            loading: false,
            bordered: true,
            max_height: 300.,
            placeholder: None,
        }
    }
}
pub struct Command {
    state: Entity<CommandState>,
    props: CommandProps,
    entries: Rc<Vec<CommandEntry>>,
    keys: Rc<Vec<Vec<String>>>,
    pending_selection: Option<Option<SharedString>>,
    event: Event<KeySelection>,
    children: NativeChildren,
    edit_seq: Rc<Cell<u32>>,
    query_seq: Rc<Cell<u32>>,
}
impl Command {
    fn entries(groups: &[CommandSection]) -> Rc<Vec<CommandEntry>> {
        Rc::new(
            groups
                .iter()
                .map(|g| {
                    let items = g.items.iter().map(|item| {
                        CommandItem::new()
                            .key(item.key.clone())
                            .label(item.label.clone())
                            .keywords(item.keywords.clone())
                            .checked(item.checked)
                            .disabled(item.disabled)
                            .when_some(item.icon.as_ref(), |v, path| {
                                v.icon(gpui_component::Icon::default().path(path.clone()))
                            })
                            .when_some(item.shortcut.as_ref(), |v, shortcut| {
                                let keystroke = gpui::Keystroke::parse(shortcut)
                                    .expect("validated command shortcut");
                                let label = item.label.clone();
                                let icon = item.icon.clone();
                                v.child(move |_, _| {
                                    gpui_component::h_flex()
                                        .w_full()
                                        .gap_2()
                                        .when_some(icon.as_ref(), |v, path| {
                                            v.child(
                                                gpui_component::Icon::default()
                                                    .path(path.clone())
                                                    .size_4(),
                                            )
                                        })
                                        .child(label.clone())
                                        .child(gpui::div().ml_auto().child(
                                            gpui_component::kbd::Kbd::new(keystroke.clone()),
                                        ))
                                })
                            })
                    });
                    CommandEntry::Group(
                        CommandGroup::new()
                            .when_some(g.label.as_ref(), |v, label| v.label(label.clone()))
                            .items(items),
                    )
                })
                .collect(),
        )
    }
    fn keys(groups: &[CommandSection]) -> Rc<Vec<Vec<String>>> {
        Rc::new(
            groups
                .iter()
                .map(|g| g.items.iter().map(|i| i.key.clone()).collect())
                .collect(),
        )
    }
}
#[crate::component]
impl NativeView for Command {
    type Props = CommandProps;
    type Event = KeySelection;
    fn event_name() -> &'static str {
        "select"
    }
    fn slots() -> &'static [&'static str] {
        &["header", "footer", "empty"]
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![
            EventDefinition::new::<DataQuery>("query"),
            EventDefinition::new::<String>("confirm"),
            EventDefinition::new::<()>("cancel"),
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
    fn validate_props(p: &CommandProps) -> Result<(), String> {
        if p.groups.len() > 256
            || p.groups.iter().map(|g| g.items.len()).sum::<usize>() > 10000
            || !p.max_height.is_finite()
            || !(40. ..=8192.).contains(&p.max_height)
        {
            return Err(
                "command requires at most 256 groups, 10000 items and maxHeight 40..8192".into(),
            );
        }
        let mut keys = HashSet::new();
        let mut groups = HashSet::new();
        for group in &p.groups {
            if group.key.is_empty() || !groups.insert(&group.key) {
                return Err("command group keys must be nonempty and unique".into());
            }
            for item in &group.items {
                if item.key.is_empty() || !keys.insert(&item.key) {
                    return Err("command item keys must be globally unique and nonempty".into());
                }
                if let Some(shortcut) = &item.shortcut {
                    gpui::Keystroke::parse(shortcut).map_err(|e| e.to_string())?;
                }
            }
        }
        if p.selected_key.as_ref().is_some_and(|k| !keys.contains(k)) {
            return Err("command selectedKey must exist in groups".into());
        }
        Ok(())
    }
    fn mount(
        props: CommandProps,
        event: Event<KeySelection>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|cx| {
            let mut state = CommandState::new(window, cx);
            if let Some(query) = &props.query {
                state.set_query(query.clone(), window, cx);
            }
            state.set_loading(props.loading, window, cx);
            state
        });
        let entries = Self::entries(&props.groups);
        let keys = Self::keys(&props.groups);
        let pending_selection = props.selected_key.clone().map(|key| Some(key.into()));
        Self {
            state,
            props,
            entries,
            keys,
            pending_selection,
            event,
            children,
            edit_seq: Rc::new(Cell::new(0)),
            query_seq: Rc::new(Cell::new(0)),
        }
    }
    fn update(&mut self, p: CommandProps, window: &mut Window, cx: &mut Context<Self>) {
        if p.groups != self.props.groups {
            self.entries = Self::entries(&p.groups);
            self.keys = Self::keys(&p.groups);
        }
        if (p.selected_key != self.props.selected_key || p.ack_edit_seq != self.props.ack_edit_seq)
            && (self.props.selected_key.is_none() && p.selected_key.is_some()
                || p.ack_edit_seq >= self.edit_seq.get())
        {
            self.pending_selection = Some(p.selected_key.clone().map(Into::into));
        }
        self.state.update(cx, |state, cx| {
            if p.query != self.props.query
                && let Some(query) = &p.query
            {
                state.set_query(query.clone(), window, cx);
            }
            if p.loading != self.props.loading {
                state.set_loading(p.loading, window, cx);
            }
        });
        self.props = p;
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("focus", |this, (): (), window, cx| {
                this.state.update(cx, |state, cx| state.focus(window, cx));
                Ok(())
            }),
            ViewCommand::new("setQuery", |this, query: String, window, cx| {
                this.state
                    .update(cx, |state, cx| state.set_query(query, window, cx));
                Ok(())
            }),
            ViewCommand::new("getSelection", |this, (): (), _, cx| {
                Ok(this.state.read(cx).selected_key().map(ToString::to_string))
            }),
        ]
    }
}
impl Render for Command {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let selection = self.event.clone();
        let seq = self.edit_seq.clone();
        let keys = self.keys.clone();
        let revision = self.props.data_revision;
        let confirm_keys = self.keys.clone();
        let confirm = self.event.related("confirm");
        let cancel = self.event.related("cancel");
        let query = self.event.related("query");
        let query_seq = self.query_seq.clone();
        let header = self.children.slot("header");
        let footer = self.children.slot("footer");
        let empty = self.children.slot("empty");
        gpui_component::command::Command::new(&self.state)
            .entries(self.entries.clone())
            .searchable(self.props.searchable)
            .filterable(self.props.filterable)
            .bordered(self.props.bordered)
            .max_h(px(self.props.max_height))
            .when_some(self.props.placeholder.as_ref(), |v, s| {
                v.placeholder(s.clone())
            })
            .when_some(self.pending_selection.take(), |v, key| v.selected_key(key))
            .when(!header.is_empty(), |v| {
                v.header(move |_, _, _| header.clone())
            })
            .when(!footer.is_empty(), |v| {
                v.footer(move |_, _, _| footer.clone())
            })
            .when(!empty.is_empty(), |v| v.empty(move |_, _, _| empty.clone()))
            .on_select(move |ix, _, _| {
                if let Some(key) = keys.get(ix.section).and_then(|g| g.get(ix.row)) {
                    let next = seq
                        .get()
                        .checked_add(1)
                        .expect("command edit sequence exhausted");
                    seq.set(next);
                    selection.emit(KeySelection {
                        value: Some(key.clone()),
                        edit_seq: next,
                        data_revision: revision,
                    });
                }
            })
            .on_confirm(move |ix, _, _| {
                if let Some(key) = confirm_keys.get(ix.section).and_then(|g| g.get(ix.row)) {
                    confirm.emit(key.clone());
                }
            })
            .on_cancel(move |_, _| cancel.emit(()))
            .on_query(move |text, _, _| {
                let id = query_seq
                    .get()
                    .checked_add(1)
                    .expect("command query sequence exhausted");
                query_seq.set(id);
                query.emit(DataQuery {
                    query: text.into(),
                    request_id: id,
                    data_revision: revision,
                });
            })
    }
}
pub(crate) fn definition() -> crate::native::ComponentDefinition {
    __native_component_Command()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    fn groups(reverse: bool) -> Vec<CommandSection> {
        let mut items: Vec<_> = ["a", "b"]
            .into_iter()
            .map(|key| CommandChoice {
                key: key.into(),
                label: key.into(),
                icon: None,
                keywords: vec![],
                shortcut: None,
                disabled: false,
                checked: false,
            })
            .collect();
        if reverse {
            items.reverse();
        }
        vec![CommandSection {
            key: "g".into(),
            label: None,
            items,
        }]
    }
    #[gpui::test]
    fn command_highlight_survives_model_reorder_and_unrelated_redraw(
        cx: &mut gpui::TestAppContext,
    ) {
        let fixture = Fixture::<Command>::new(
            CommandProps {
                groups: groups(false),
                selected_key: Some("b".into()),
                ..Default::default()
            },
            cx,
        );
        cx.update_window(fixture.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx)
        })
        .unwrap();
        let id = fixture.update(cx, |v, _, cx| {
            assert_eq!(
                v.state.read(cx).selected_key().map(|s| s.as_ref()),
                Some("b")
            );
            v.state.entity_id()
        });
        fixture.update(cx, |v, window, cx| {
            v.update(
                CommandProps {
                    groups: groups(true),
                    selected_key: Some("b".into()),
                    data_revision: 1,
                    ..Default::default()
                },
                window,
                cx,
            )
        });
        cx.update_window(fixture.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx)
        })
        .unwrap();
        fixture.update(cx, |v, _, cx| {
            assert_eq!(v.state.entity_id(), id);
            assert_eq!(
                v.state.read(cx).selected_key().map(|s| s.as_ref()),
                Some("b")
            );
            assert_eq!(v.state.read(cx).selected_index().unwrap().row, 0);
        });
    }
}
