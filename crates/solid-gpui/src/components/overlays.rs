//! Root-owned modals. Solid owns `open`; each mounted view owns one native token.
use super::ButtonVariant;
use crate::native::{
    ComponentDefinition, Event, EventDefinition, NativeChildren, NativeView, TS, ViewCommand,
};
use gpui::{App, Context, IntoElement, ParentElement, Render, WeakEntity, Window, px};
use gpui_component::dialog::DialogButtonProps;
use gpui_component::{OverlayCloseReason, OverlayToken, Root};

#[crate::native_type]
#[derive(Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct ModalButtons {
    pub ok_text: Option<String>,
    pub cancel_text: Option<String>,
    pub ok_variant: ButtonVariant,
    pub cancel_variant: ButtonVariant,
    pub show_cancel: bool,
    pub close_on_ok: bool,
    pub close_on_cancel: bool,
}
impl Default for ModalButtons {
    fn default() -> Self {
        Self {
            ok_text: None,
            cancel_text: None,
            ok_variant: ButtonVariant::Primary,
            cancel_variant: ButtonVariant::Default,
            show_cancel: false,
            close_on_ok: true,
            close_on_cancel: true,
        }
    }
}
impl ModalButtons {
    fn native(&self) -> DialogButtonProps {
        let mut p = DialogButtonProps::default()
            .ok_variant(self.ok_variant.into())
            .cancel_variant(self.cancel_variant.into())
            .show_cancel(self.show_cancel);
        if let Some(s) = &self.ok_text {
            p = p.ok_text(s.clone());
        }
        if let Some(s) = &self.cancel_text {
            p = p.cancel_text(s.clone());
        }
        p
    }
}
macro_rules! dialog_props {
    ($name:ident { $($field:ident : $ty:ty = $default:expr),* $(,)? }) => {
        #[crate::native_type]
        #[derive(Clone)]
        #[serde(default, rename_all = "camelCase")]
        pub struct $name { pub open: bool, pub title: Option<String>, pub width: f32, pub close_button: bool, pub keyboard: bool, pub buttons: ModalButtons, $(pub $field: $ty,)* }
        impl Default for $name { fn default() -> Self { Self { open: false, title: None, width: 480., close_button: true, keyboard: true, buttons: ModalButtons::default(), $($field: $default,)* } } }
    }
}
dialog_props!(DialogProps { max_width: Option<f32> = None, margin_top: Option<f32> = None, overlay: bool = true, overlay_closable: bool = true });
dialog_props!(AlertDialogProps { description: Option<String> = None });
#[crate::native_type]
#[derive(Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}
impl From<Side> for gpui_component::Placement {
    fn from(v: Side) -> Self {
        match v {
            Side::Left => Self::Left,
            Side::Right => Self::Right,
            Side::Top => Self::Top,
            Side::Bottom => Self::Bottom,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy)]
#[serde(tag = "unit", content = "value", rename_all = "lowercase")]
pub enum SheetSize {
    Px(f32),
    Relative(f32),
}
impl Default for SheetSize {
    fn default() -> Self {
        Self::Px(350.)
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct SheetProps {
    pub open: bool,
    pub title: Option<String>,
    pub placement: Side,
    pub size: SheetSize,
    pub resizable: bool,
    pub overlay: bool,
    pub overlay_closable: bool,
}
impl Default for SheetProps {
    fn default() -> Self {
        Self {
            open: false,
            title: None,
            placement: Side::Right,
            size: SheetSize::default(),
            resizable: true,
            overlay: true,
            overlay_closable: true,
        }
    }
}

#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModalCloseReason {
    Ok,
    Cancel,
    Dismissed,
    Programmatic,
    Replaced,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModalOpenChange {
    pub open: bool,
    pub session_id: u32,
    pub reason: ModalCloseReason,
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModalActionKind {
    Ok,
    Cancel,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModalAction {
    pub kind: ModalActionKind,
    pub request_id: u32,
    pub session_id: u32,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolveModal {
    pub request_id: u32,
    pub close: bool,
}

trait ModalKind: Sized + 'static {
    type Props: Clone + serde::de::DeserializeOwned + TS + 'static;
    const ACTIONS: bool = true;
    fn slots() -> &'static [&'static str] {
        &["title", "footer"]
    }
    fn open(p: &Self::Props) -> bool;
    fn buttons(p: &Self::Props) -> Option<&ModalButtons>;
    fn validate(p: &Self::Props) -> Result<(), String>;
    fn attach(
        props: &Self::Props,
        view: WeakEntity<Modal<Self>>,
        session: u32,
        window: &mut Window,
        cx: &mut App,
    ) -> OverlayToken;
    fn refresh(p: &Self::Props, token: &OverlayToken, cx: &mut App) {
        let _ = p;
        token.refresh(cx);
    }
}
struct Dialog;
struct AlertDialog;
struct Sheet;
struct Modal<M: ModalKind> {
    props: M::Props,
    children: NativeChildren,
    event: Event<ModalOpenChange>,
    token: Option<OverlayToken>,
    session: u32,
    request_seq: u32,
    pending: Option<ModalAction>,
    closing_reason: Option<ModalCloseReason>,
    mounted: bool,
}
impl<M: ModalKind> Modal<M> {
    fn attach(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.session = self
            .session
            .checked_add(1)
            .expect("modal session exhausted");
        self.pending = None;
        self.closing_reason = None;
        self.token = Some(M::attach(
            &self.props,
            cx.entity().downgrade(),
            self.session,
            window,
            cx,
        ));
    }
    fn close(&mut self, window: &mut Window, cx: &mut App) {
        self.pending = None;
        if let Some(token) = &self.token {
            token.close(window, cx);
        }
    }
    fn action(&mut self, kind: ModalActionKind, session: u32) -> bool {
        if session != self.session || !self.token.as_ref().is_some_and(OverlayToken::is_open) {
            return false;
        }
        let Some(buttons) = M::buttons(&self.props) else {
            return false;
        };
        let close = match kind {
            ModalActionKind::Ok => buttons.close_on_ok,
            ModalActionKind::Cancel => buttons.close_on_cancel,
        };
        self.request_seq = self
            .request_seq
            .checked_add(1)
            .expect("modal request exhausted");
        let action = ModalAction {
            kind,
            request_id: self.request_seq,
            session_id: session,
        };
        self.pending = (!close).then_some(action.clone());
        self.closing_reason = close.then_some(match kind {
            ModalActionKind::Ok => ModalCloseReason::Ok,
            ModalActionKind::Cancel => ModalCloseReason::Cancel,
        });
        self.event.related("action").emit(action);
        close
    }
    fn closed(&mut self, session: u32, reason: OverlayCloseReason) {
        if self.session != session {
            return;
        }
        self.pending = None;
        let reason = self.closing_reason.take().unwrap_or(match reason {
            OverlayCloseReason::Dismissed => ModalCloseReason::Dismissed,
            OverlayCloseReason::Programmatic => ModalCloseReason::Programmatic,
            OverlayCloseReason::Replaced => ModalCloseReason::Replaced,
        });
        self.event.emit(ModalOpenChange {
            open: false,
            session_id: session,
            reason,
        });
    }
    fn resolve(
        &mut self,
        request: ResolveModal,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<(), String> {
        let pending = self
            .pending
            .as_ref()
            .filter(|p| {
                p.request_id == request.request_id
                    && p.session_id == self.session
                    && self.token.as_ref().is_some_and(OverlayToken::is_open)
            })
            .ok_or("modal request is retired")?
            .clone();
        self.pending = None;
        if request.close {
            self.closing_reason = Some(match pending.kind {
                ModalActionKind::Ok => ModalCloseReason::Ok,
                ModalActionKind::Cancel => ModalCloseReason::Cancel,
            });
            self.close(window, cx);
        }
        Ok(())
    }
}
fn on_closed<M: ModalKind>(
    view: WeakEntity<Modal<M>>,
    session: u32,
) -> impl Fn(OverlayCloseReason, &mut Window, &mut App) {
    move |reason, _, cx| {
        let _ = view.update(cx, |view, _| view.closed(session, reason));
    }
}
fn on_action<M: ModalKind>(
    view: WeakEntity<Modal<M>>,
    session: u32,
    kind: ModalActionKind,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) -> bool {
    move |_, _, cx| {
        view.update(cx, |view, _| view.action(kind, session))
            .unwrap_or(false)
    }
}
fn extent(v: f32) -> Result<(), String> {
    if v.is_finite() && (0.0..=1_000_000.0).contains(&v) {
        Ok(())
    } else {
        Err("modal dimensions must be finite and between 0 and 1000000".into())
    }
}
impl ModalKind for Dialog {
    type Props = DialogProps;
    fn open(p: &Self::Props) -> bool {
        p.open
    }
    fn buttons(p: &Self::Props) -> Option<&ModalButtons> {
        Some(&p.buttons)
    }
    fn validate(p: &Self::Props) -> Result<(), String> {
        extent(p.width)?;
        if let Some(v) = p.max_width {
            extent(v)?;
        }
        if let Some(v) = p.margin_top {
            extent(v)?;
        }
        Ok(())
    }
    fn attach(
        _: &Self::Props,
        view: WeakEntity<Modal<Self>>,
        session: u32,
        window: &mut Window,
        cx: &mut App,
    ) -> OverlayToken {
        let closed = on_closed(view.clone(), session);
        Root::update(window, cx, |root, window, cx| {
            root.open_dialog_owned(
                move |dialog, _, cx| {
                    let Some(entity) = view.upgrade() else {
                        return dialog;
                    };
                    let model = entity.read(cx);
                    let p = &model.props;
                    let mut d = dialog
                        .width(px(p.width))
                        .close_button(p.close_button)
                        .keyboard(p.keyboard)
                        .overlay(p.overlay)
                        .overlay_closable(p.overlay_closable)
                        .button_props(p.buttons.native())
                        .footer(p.buttons.native().render_footer())
                        .child(model.children.content())
                        .on_ok(on_action(view.clone(), session, ModalActionKind::Ok))
                        .on_cancel(on_action(view.clone(), session, ModalActionKind::Cancel));
                    if let Some(v) = p.max_width {
                        d = d.max_w(px(v));
                    }
                    if let Some(v) = p.margin_top {
                        d = d.margin_top(px(v));
                    }
                    let title = model.children.slot("title");
                    if !title.is_empty() {
                        d = d.title(title);
                    } else if let Some(v) = &p.title {
                        d = d.title(v.clone());
                    }
                    let footer = model.children.slot("footer");
                    if !footer.is_empty() {
                        d = d.footer(footer);
                    }
                    d
                },
                closed,
                window,
                cx,
            )
        })
    }
}
impl ModalKind for AlertDialog {
    type Props = AlertDialogProps;
    fn slots() -> &'static [&'static str] {
        &["title", "footer", "icon"]
    }
    fn open(p: &Self::Props) -> bool {
        p.open
    }
    fn buttons(p: &Self::Props) -> Option<&ModalButtons> {
        Some(&p.buttons)
    }
    fn validate(p: &Self::Props) -> Result<(), String> {
        extent(p.width)
    }
    fn attach(
        _: &Self::Props,
        view: WeakEntity<Modal<Self>>,
        session: u32,
        window: &mut Window,
        cx: &mut App,
    ) -> OverlayToken {
        let closed = on_closed(view.clone(), session);
        Root::update(window, cx, |root, window, cx| {
            root.open_alert_dialog_owned(
                move |alert, _, cx| {
                    let Some(entity) = view.upgrade() else {
                        return alert;
                    };
                    let model = entity.read(cx);
                    let p = &model.props;
                    let mut d = alert
                        .width(px(p.width))
                        .close_button(p.close_button)
                        .keyboard(p.keyboard)
                        .button_props(p.buttons.native())
                        .child(model.children.content())
                        .on_ok(on_action(view.clone(), session, ModalActionKind::Ok))
                        .on_cancel(on_action(view.clone(), session, ModalActionKind::Cancel));
                    let title = model.children.slot("title");
                    if !title.is_empty() {
                        d = d.title(title);
                    } else if let Some(v) = &p.title {
                        d = d.title(v.clone());
                    }
                    if let Some(v) = &p.description {
                        d = d.description(v.clone());
                    }
                    let icon = model.children.slot("icon");
                    if !icon.is_empty() {
                        d = d.icon(icon);
                    }
                    let footer = model.children.slot("footer");
                    if !footer.is_empty() {
                        d = d.footer(footer);
                    }
                    d
                },
                closed,
                window,
                cx,
            )
        })
    }
}
impl ModalKind for Sheet {
    type Props = SheetProps;
    const ACTIONS: bool = false;
    fn open(p: &Self::Props) -> bool {
        p.open
    }
    fn buttons(_: &Self::Props) -> Option<&ModalButtons> {
        None
    }
    fn validate(p: &Self::Props) -> Result<(), String> {
        match p.size {
            SheetSize::Px(v) => extent(v),
            SheetSize::Relative(v) if v.is_finite() && (0.0..=1.0).contains(&v) => Ok(()),
            _ => Err("relative sheet size must be between 0 and 1".into()),
        }
    }
    fn attach(
        props: &Self::Props,
        view: WeakEntity<Modal<Self>>,
        session: u32,
        window: &mut Window,
        cx: &mut App,
    ) -> OverlayToken {
        let placement = props.placement;
        let closed = on_closed(view.clone(), session);
        Root::update(window, cx, |root, window, cx| {
            root.open_sheet_owned(
                placement.into(),
                move |sheet, _, cx| {
                    let Some(entity) = view.upgrade() else {
                        return sheet;
                    };
                    let model = entity.read(cx);
                    let p = &model.props;
                    let size: gpui::DefiniteLength = match p.size {
                        SheetSize::Px(v) => px(v).into(),
                        SheetSize::Relative(v) => gpui::relative(v),
                    };
                    let mut s = sheet
                        .size(size)
                        .resizable(p.resizable)
                        .overlay(p.overlay)
                        .overlay_closable(p.overlay_closable)
                        .child(model.children.content());
                    let title = model.children.slot("title");
                    if !title.is_empty() {
                        s = s.title(title);
                    } else if let Some(v) = &p.title {
                        s = s.title(v.clone());
                    }
                    let footer = model.children.slot("footer");
                    if !footer.is_empty() {
                        s = s.footer(footer);
                    }
                    s
                },
                closed,
                window,
                cx,
            )
        })
    }
    fn refresh(p: &Self::Props, token: &OverlayToken, cx: &mut App) {
        token.set_sheet_placement(p.placement.into(), cx);
        token.refresh(cx);
    }
}
impl<M: ModalKind> NativeView for Modal<M> {
    type Props = M::Props;
    type Event = ModalOpenChange;
    fn event_name() -> &'static str {
        "openChange"
    }
    fn accepts_children() -> bool {
        true
    }
    fn slots() -> &'static [&'static str] {
        M::slots()
    }
    fn additional_events() -> Vec<EventDefinition> {
        if M::ACTIONS {
            vec![EventDefinition::new::<ModalAction>("action")]
        } else {
            Vec::new()
        }
    }
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        M::validate(p)
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Component construction can precede installation of the window Root.
        // Deferred mounting also ensures the entity exists before builders read it.
        cx.defer_in(window, |view, window, cx| {
            if view.mounted && M::open(&view.props) && view.token.is_none() {
                view.attach(window, cx);
            }
        });
        Self {
            props,
            children,
            event,
            token: None,
            session: 0,
            request_seq: 0,
            pending: None,
            closing_reason: None,
            mounted: true,
        }
    }
    fn update(&mut self, props: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        let was_open = M::open(&self.props);
        let open = M::open(&props);
        self.props = props;
        if open && !was_open {
            self.attach(window, cx);
        } else if !open {
            self.close(window, cx);
        } else if let Some(token) = &self.token {
            M::refresh(&self.props, token, cx);
        }
    }
    fn unmount(&mut self, window: &mut Window, cx: &mut App) {
        self.mounted = false;
        self.close(window, cx);
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        let mut commands = vec![ViewCommand::new(
            "isOpen",
            |view: &mut Self, _: (), _, _| {
                Ok(view.token.as_ref().is_some_and(OverlayToken::is_open))
            },
        )];
        if M::ACTIONS {
            commands.push(ViewCommand::new(
                "resolve",
                |view: &mut Self, request: ResolveModal, window, cx| {
                    view.resolve(request, window, cx)
                },
            ));
        }
        commands
    }
}
impl<M: ModalKind> Render for Modal<M> {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}
pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::view::<Modal<Dialog>>("Dialog"),
        ComponentDefinition::view::<Modal<AlertDialog>>("AlertDialog"),
        ComponentDefinition::view::<Modal<Sheet>>("Sheet"),
    ]
    .into_iter()
    .map(|d| d.with_contract(include_str!("overlays.rs")))
    .collect()
}

#[crate::native_module(name = "gpui-component")]
mod parts {
    use crate::native::ElementContext;
    use gpui::{IntoElement, ParentElement, Styled};
    use gpui_component::dialog;
    #[component]
    fn dialog_content(cx: &mut ElementContext) -> impl IntoElement + Styled {
        dialog::DialogContent::new().children(cx.children())
    }
    #[component]
    fn dialog_header(cx: &mut ElementContext) -> impl IntoElement + Styled {
        dialog::DialogHeader::new().children(cx.children())
    }
    #[component]
    fn dialog_title(cx: &mut ElementContext) -> impl IntoElement + Styled {
        dialog::DialogTitle::new().children(cx.children())
    }
    #[component]
    fn dialog_description(cx: &mut ElementContext) -> impl IntoElement + Styled {
        dialog::DialogDescription::new().children(cx.children())
    }
    #[component]
    fn dialog_footer(cx: &mut ElementContext) -> impl IntoElement + Styled {
        dialog::DialogFooter::new().children(cx.children())
    }
    #[component]
    fn dialog_close(cx: &mut ElementContext) -> impl IntoElement {
        dialog::DialogClose::new().children(cx.children())
    }
    #[component]
    fn dialog_action(cx: &mut ElementContext) -> impl IntoElement {
        dialog::DialogAction::new().children(cx.children())
    }
}
pub(super) use parts::native_module;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    #[gpui::test]
    fn modal_requests_are_session_scoped_and_unmount_closes_only_its_owner(
        cx: &mut gpui::TestAppContext,
    ) {
        let fixture = Fixture::<Modal<Dialog>>::new(
            DialogProps {
                open: true,
                buttons: ModalButtons {
                    close_on_ok: false,
                    ..Default::default()
                },
                ..Default::default()
            },
            cx,
        );
        cx.run_until_parked();
        fixture.update(cx, |view, window, cx| {
            assert!(view.token.as_ref().unwrap().is_open());
            assert!(!view.action(ModalActionKind::Ok, view.session));
            let stale_request = view.pending.as_ref().unwrap().request_id;
            let mut p = view.props.clone();
            p.open = false;
            view.update(p.clone(), window, cx);
            p.open = true;
            view.update(p, window, cx);
            assert!(
                view.resolve(
                    ResolveModal {
                        request_id: stale_request,
                        close: true
                    },
                    window,
                    cx
                )
                .is_err()
            );
            assert!(view.token.as_ref().unwrap().is_open());
            assert!(!view.action(ModalActionKind::Ok, view.session));
            let request_id = view.pending.as_ref().unwrap().request_id;
            view.resolve(
                ResolveModal {
                    request_id,
                    close: true,
                },
                window,
                cx,
            )
            .unwrap();
            let mut p = view.props.clone();
            p.width = 600.;
            view.update(p.clone(), window, cx);
            assert!(
                !view.token.as_ref().unwrap().is_open(),
                "an unrelated prop must not reopen a dismissed controlled modal"
            );
            p.open = false;
            view.update(p.clone(), window, cx);
            p.open = true;
            view.update(p, window, cx);
            let other = Root::update(window, cx, |root, window, cx| {
                root.open_dialog_owned(|d, _, _| d.title("other owner"), |_, _, _| {}, window, cx)
            });
            view.unmount(window, cx);
            assert!(!view.token.as_ref().unwrap().is_open());
            assert!(other.is_open());
            other.close(window, cx);
        });
        cx.run_until_parked();
        let sheet = Fixture::<Modal<Sheet>>::new(
            SheetProps {
                open: true,
                ..Default::default()
            },
            cx,
        );
        cx.run_until_parked();
        sheet.update(cx, |view, window, cx| {
            let token = view.token.as_ref().unwrap().clone();
            let mut props = view.props.clone();
            props.placement = Side::Left;
            view.update(props, window, cx);
            assert!(token.is_open());
            view.unmount(window, cx);
            assert!(!token.is_open());
        });
    }
}
