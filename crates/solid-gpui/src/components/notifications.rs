use super::{ButtonVariant, popups::PopupAnchor};
use crate::native::{
    ComponentDefinition, Event, EventDefinition, NativeChildren, NativeView, ViewCommand,
};
use gpui::{App, Context, Entity, IntoElement, Render, WeakEntity, Window};
use gpui_component::{
    Root,
    button::{Button, ButtonVariants},
    notification::{
        Notification as NativeNotification, NotificationDelivery as NativeDelivery,
        NotificationType as NativeType,
    },
};

#[crate::native_type]
#[derive(Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationKind {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}
impl From<NotificationKind> for NativeType {
    fn from(v: NotificationKind) -> Self {
        match v {
            NotificationKind::Info => Self::Info,
            NotificationKind::Success => Self::Success,
            NotificationKind::Warning => Self::Warning,
            NotificationKind::Error => Self::Error,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum NotificationDelivery {
    #[default]
    InApp,
    System,
    InAppAndSystem,
}
impl From<NotificationDelivery> for NativeDelivery {
    fn from(v: NotificationDelivery) -> Self {
        match v {
            NotificationDelivery::InApp => Self::InApp,
            NotificationDelivery::System => Self::System,
            NotificationDelivery::InAppAndSystem => Self::InAppAndSystem,
        }
    }
}
#[crate::native_type]
#[derive(Clone, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct NotificationProps {
    pub open: bool,
    pub title: Option<String>,
    pub message: Option<String>,
    pub kind: NotificationKind,
    pub placement: PopupAnchor,
    pub delivery: NotificationDelivery,
    pub autohide: bool,
    pub action_label: Option<String>,
    pub action_variant: ButtonVariant,
    pub close_on_action: bool,
}
impl Default for NotificationProps {
    fn default() -> Self {
        Self {
            open: false,
            title: None,
            message: None,
            kind: NotificationKind::Info,
            placement: PopupAnchor::TopRight,
            delivery: NotificationDelivery::InApp,
            autohide: true,
            action_label: None,
            action_variant: ButtonVariant::Default,
            close_on_action: true,
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotificationClosed {
    pub session_id: u32,
}
struct Notification {
    props: NotificationProps,
    children: NativeChildren,
    event: Event<NotificationClosed>,
    root: Option<WeakEntity<Root>>,
    note: Option<Entity<NativeNotification>>,
    key: String,
    session: u32,
    live: bool,
    mounted: bool,
}
impl Notification {
    fn build(&self, cx: &Context<Self>) -> NativeNotification {
        let weak = cx.entity().downgrade();
        let event = self.event.clone();
        let session = self.session;
        let mut note = NativeNotification::new()
            .id1::<Self>(self.key.clone())
            .with_type(self.props.kind.into())
            .placement(self.props.placement.into())
            .delivery(self.props.delivery.into())
            .autohide(self.props.autohide)
            .on_click(move |_, _, _| event.related::<()>("click").emit(()))
            .on_close(move |_, cx| {
                let _ = weak.update(cx, |view, _| {
                    if view.session == session && view.live {
                        view.live = false;
                        view.event.emit(NotificationClosed {
                            session_id: session,
                        });
                    }
                });
            });
        if let Some(title) = &self.props.title {
            note = note.title(title.clone());
        }
        if let Some(message) = &self.props.message {
            note = note.message(message.clone());
        }
        let content = self.children.content();
        if !content.is_empty() {
            note = note.content(move |_, _, _| content.clone().into_any_element());
        }
        if let Some(label) = &self.props.action_label {
            let label = label.clone();
            let variant = self.props.action_variant;
            let close = self.props.close_on_action;
            let event = self.event.clone();
            note = note.action(move |_, _, cx| {
                let event = event.clone();
                Button::new("action")
                    .label(label.clone())
                    .with_variant(variant.into())
                    .on_click(cx.listener(move |note, _, window, cx| {
                        event.related::<()>("action").emit(());
                        if close {
                            note.dismiss(window, cx);
                        }
                    }))
            });
        }
        note
    }
    fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.session = self
            .session
            .checked_add(1)
            .expect("notification session exhausted");
        self.key = format!("{}:{}", cx.entity_id().as_u64(), self.session);
        self.live = true;
        let note = self.build(cx);
        self.note = Root::update(window, cx, |root, window, cx| {
            self.root = Some(cx.entity().downgrade());
            root.push_notification(note, window, cx);
            root.notification
                .read(cx)
                .notification1::<Self>(self.key.clone())
        });
    }
    fn close(&mut self, window: &mut Window, cx: &mut App) {
        if let Some(root) = &self.root {
            let key = self.key.clone();
            let _ = root.update(cx, |root, cx| {
                root.remove_notification1::<Self>(key, window, cx)
            });
        }
        // System-only notifications have no toast exit callback.
        if self.props.delivery == NotificationDelivery::System && self.live {
            self.live = false;
            self.event.emit(NotificationClosed {
                session_id: self.session,
            });
        }
    }
}
impl NativeView for Notification {
    type Props = NotificationProps;
    type Event = NotificationClosed;
    fn event_name() -> &'static str {
        "close"
    }
    fn accepts_children() -> bool {
        true
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![
            EventDefinition::new::<()>("action"),
            EventDefinition::new::<()>("click"),
        ]
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.defer_in(window, |view, window, cx| {
            if view.mounted && view.props.open && view.session == 0 {
                view.show(window, cx);
            }
        });
        Self {
            props,
            children,
            event,
            root: None,
            note: None,
            key: String::new(),
            session: 0,
            live: false,
            mounted: true,
        }
    }
    fn update(&mut self, props: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        let reopen = props.open && (!self.props.open || props.delivery != self.props.delivery);
        let changed = props != self.props;
        if !props.open || reopen {
            self.close(window, cx);
        }
        self.props = props;
        if reopen {
            self.show(window, cx);
        } else if self.live && self.props.open {
            if changed {
                let note = self.build(cx);
                if let Some(root) = &self.root {
                    let _ = root.update(cx, |root, cx| {
                        if self.props.delivery == NotificationDelivery::System {
                            root.push_notification(note, window, cx);
                        } else {
                            root.notification.update(cx, |list, cx| {
                                list.revise(note, window, cx);
                            });
                        }
                    });
                }
            } else if let Some(note) = &self.note {
                note.update(cx, |_, cx| cx.notify());
            }
        }
    }
    fn unmount(&mut self, window: &mut Window, cx: &mut App) {
        self.mounted = false;
        self.close(window, cx);
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![ViewCommand::new(
            "dismiss",
            |view: &mut Self, _: (), window, cx| {
                view.close(window, cx);
                Ok(())
            },
        )]
    }
}
impl Render for Notification {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}
pub(super) fn definition() -> ComponentDefinition {
    ComponentDefinition::view::<Notification>("Notification")
        .with_contract(include_str!("notifications.rs"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    #[gpui::test]
    fn updating_toast_content_preserves_native_entity_and_owner_cleanup(
        cx: &mut gpui::TestAppContext,
    ) {
        let fixture = Fixture::<Notification>::new(
            NotificationProps {
                open: true,
                autohide: false,
                message: Some("before".into()),
                ..Default::default()
            },
            cx,
        );
        cx.run_until_parked();
        fixture.update(cx, |view, window, cx| {
            let id = view.note.as_ref().unwrap().entity_id();
            let key = view.key.clone();
            let mut props = view.props.clone();
            props.message = Some("after".into());
            view.update(props, window, cx);
            assert_eq!(view.note.as_ref().unwrap().entity_id(), id);
            assert_eq!(view.key, key);
            Root::update(window, cx, |root, window, cx| {
                root.push_notification(
                    NativeNotification::new()
                        .id1::<Notification>("another")
                        .autohide(false),
                    window,
                    cx,
                )
            });
            view.unmount(window, cx);
            assert!(
                Root::read(window, cx)
                    .notification
                    .read(cx)
                    .notification1::<Notification>("another")
                    .is_some()
            );
        });
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(300));
        cx.run_until_parked();
        fixture.update(cx, |view, _, _| assert!(!view.live));
    }
}
