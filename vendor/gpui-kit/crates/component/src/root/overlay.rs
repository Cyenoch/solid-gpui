use super::{ActiveDialog, ActiveSheet, Root};
use crate::{
    Placement,
    dialog::{ANIMATION_DURATION, AlertDialog, Dialog},
    sheet::Sheet,
};
use gpui::{App, Context, WeakEntity, WeakFocusHandle, Window};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayCloseReason {
    Dismissed,
    Programmatic,
    Replaced,
}

type CloseCallback = Rc<dyn Fn(OverlayCloseReason, &mut Window, &mut App)>;

#[derive(Clone)]
pub(super) struct OverlayLifetime {
    pub(super) id: u64,
    open: Rc<Cell<bool>>,
    on_closed: CloseCallback,
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        AppContext, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render, Styled,
        div, px,
    };
    use std::{cell::RefCell, collections::HashMap, time::Duration};
    struct Surface {
        focus: FocusHandle,
    }
    impl Render for Surface {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .track_focus(&self.focus)
                .children(Root::render_sheet_layer(window, cx))
                .children(Root::render_dialog_layer(window, cx))
        }
    }
    fn fixture(
        cx: &mut gpui::TestAppContext,
    ) -> (
        gpui::Entity<Root>,
        &mut gpui::VisualTestContext,
        FocusHandle,
    ) {
        cx.update(crate::init);
        let focus = cx.update(|cx| cx.focus_handle());
        let (root, cx) = cx.add_window_view({
            let focus = focus.clone();
            move |window, cx| {
                let view = cx.new(|_| Surface { focus });
                Root::new(view, window, cx)
            }
        });
        cx.update(|window, cx| focus.focus(window, cx));
        (root, cx, focus)
    }
    #[gpui::test]
    fn token_retirement_preserves_neighbors_and_pending_dialog_focus(
        cx: &mut gpui::TestAppContext,
    ) {
        let (root, cx, background) = fixture(cx);
        let closed = Rc::new(RefCell::new(Vec::new()));
        let (a, b, a_focus) = cx.update(|window, cx| {
            root.update(cx, |root, cx| {
                let a = root.open_dialog_owned(|d, _, _| d.title("A"), |_, _, _| {}, window, cx);
                let a_focus = root.active_dialogs[0].focus_handle.clone();
                let b = root.open_dialog_owned(|d, _, _| d.title("B"), |_, _, _| {}, window, cx);
                (a, b, a_focus)
            })
        });
        cx.update(|window, cx| {
            root.update(cx, |root, cx| {
                assert!(root.close_overlay(b.id, OverlayCloseReason::Dismissed, true, window, cx));
            })
        });
        let sheet = cx.update(|window, cx| {
            root.update(cx, |root, cx| {
                root.open_sheet_owned(
                    Placement::Right,
                    |s, _, _| s.title("S1"),
                    {
                        let closed = closed.clone();
                        move |reason, _, _| closed.borrow_mut().push(reason)
                    },
                    window,
                    cx,
                )
            })
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(300));
        cx.run_until_parked();
        assert!(cx.update(|window, _| a_focus.is_focused(window)));
        assert!(!b.is_open());
        let newer = cx.update(|window, cx| {
            root.update(cx, |root, cx| {
                root.open_sheet_owned(
                    Placement::Left,
                    |s, _, _| s.title("S2"),
                    |_, _, _| {},
                    window,
                    cx,
                )
            })
        });
        assert!(!sheet.is_open());
        cx.update(|window, cx| {
            assert!(!sheet.close(window, cx));
            assert!(a.close(window, cx));
        });
        assert!(newer.is_open());
        cx.update(|window, cx| {
            assert!(newer.close(window, cx));
            assert!(background.is_focused(window));
        });
        cx.run_until_parked();
        assert_eq!(&*closed.borrow(), &[OverlayCloseReason::Replaced]);
    }

    #[gpui::test]
    fn rendered_dialog_identity_survives_removal_and_confirm_targets_original_layer(
        cx: &mut gpui::TestAppContext,
    ) {
        let (root, cx, _) = fixture(cx);
        let observed = Rc::new(RefCell::new(HashMap::new()));
        let build =
            |key: &'static str, observed: Rc<RefCell<HashMap<&'static str, gpui::EntityId>>>| {
                move |d: Dialog, window: &mut Window, cx: &mut App| {
                    let state = window.use_keyed_state("same-local-state", cx, |_, _| ());
                    let id = state.entity_id();
                    observed.borrow_mut().insert(key, id);
                    d.width(px(300.))
                        .title(key)
                        .child(div().h(px(40.)).child(key))
                }
            };
        let (a, b) = cx.update(|window, cx| {
            root.update(cx, |root, cx| {
                let a =
                    root.open_dialog_owned(build("A", observed.clone()), |_, _, _| {}, window, cx);
                let b =
                    root.open_dialog_owned(build("B", observed.clone()), |_, _, _| {}, window, cx);
                (a, b)
            })
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let first = observed.borrow().clone();
        assert_ne!(first["A"], first["B"]);
        cx.update(|window, cx| {
            a.close(window, cx);
            window.draw(cx).clear(cx);
        });
        assert_eq!(observed.borrow()["B"], first["B"]);
        assert!(b.is_open());
        cx.update(|window, cx| {
            b.close(window, cx);
        });
        let replacement = Rc::new(RefCell::new(None));
        let original = cx.update(|window, cx| {
            root.update(cx, |root, cx| {
                root.open_dialog_owned(
                    {
                        let replacement = replacement.clone();
                        move |d, _, _| {
                            d.title("Confirm then open").on_ok({
                                let replacement = replacement.clone();
                                move |_, window, cx| {
                                    let token = Root::update(window, cx, |root, window, cx| {
                                        root.open_dialog_owned(
                                            |d, _, _| d.title("New"),
                                            |_, _, _| {},
                                            window,
                                            cx,
                                        )
                                    });
                                    *replacement.borrow_mut() = Some(token);
                                    true
                                }
                            })
                        }
                    },
                    |_, _, _| {},
                    window,
                    cx,
                )
            })
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.dispatch_action(crate::dialog::Confirm { secondary: false });
        cx.run_until_parked();
        assert!(!original.is_open());
        assert!(replacement.borrow().as_ref().unwrap().is_open());
        assert_eq!(root.read_with(cx, |r, _| r.active_dialogs.len()), 1);
    }
}
impl OverlayLifetime {
    pub(super) fn is_open(&self) -> bool {
        self.open.get()
    }
    fn retire(&self, reason: OverlayCloseReason, window: &mut Window, cx: &mut App) {
        if self.open.replace(false) {
            let callback = self.on_closed.clone();
            // Owners can be updating when they close their layer. Deliver after
            // releasing both entities; a callback may open another modal.
            window.defer(cx, move |window, cx| callback(reason, window, cx));
        }
    }
}

/// An identity boundary with the child's own layout. Modal positions and draw
/// priorities may change without moving their native element state to a new path.
pub(super) struct OverlayElement {
    id: gpui::ElementId,
    child: gpui::AnyElement,
}
impl OverlayElement {
    pub(super) fn new(id: u64, child: impl gpui::IntoElement) -> Self {
        Self {
            id: ("overlay", id).into(),
            child: child.into_any_element(),
        }
    }
}
impl gpui::IntoElement for OverlayElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl gpui::Element for OverlayElement {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<gpui::ElementId> {
        Some(self.id.clone())
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

/// A single Root-owned overlay. Clones address the same layer; closing a retired
/// token is harmless and can never remove a newer layer or another owner.
#[derive(Clone)]
pub struct OverlayToken {
    root: WeakEntity<Root>,
    id: u64,
    open: Rc<Cell<bool>>,
}
impl OverlayToken {
    pub fn is_open(&self) -> bool {
        self.open.get() && self.root.upgrade().is_some()
    }
    pub fn close(&self, window: &mut Window, cx: &mut App) -> bool {
        self.root
            .update(cx, |root, cx| {
                if root.window_id != window.window_handle().window_id() {
                    return false;
                }
                root.close_overlay(self.id, OverlayCloseReason::Programmatic, false, window, cx)
            })
            .unwrap_or(false)
    }
    /// Notify after changing the data read by the layer's builder.
    pub fn refresh(&self, cx: &mut App) {
        if self.is_open() {
            let _ = self.root.update(cx, |_, cx| cx.notify());
        }
    }
    pub fn set_sheet_placement(&self, placement: Placement, cx: &mut App) {
        let _ = self.root.update(cx, |root, cx| {
            if let Some(sheet) = root
                .active_sheet
                .as_mut()
                .filter(|s| s.lifetime.id == self.id)
            {
                if sheet.placement != placement {
                    sheet.placement = placement;
                    cx.notify();
                }
            }
        });
    }
}

impl Root {
    fn overlay_lifetime(
        &mut self,
        on_closed: CloseCallback,
        cx: &Context<Self>,
    ) -> (OverlayLifetime, OverlayToken) {
        self.next_overlay_id = self
            .next_overlay_id
            .checked_add(1)
            .expect("overlay id exhausted");
        let lifetime = OverlayLifetime {
            id: self.next_overlay_id,
            open: Rc::new(Cell::new(true)),
            on_closed,
        };
        let token = OverlayToken {
            root: cx.entity().downgrade(),
            id: lifetime.id,
            open: lifetime.open.clone(),
        };
        (lifetime, token)
    }

    fn take_previous_focus(&mut self, window: &Window, cx: &App) -> Option<WeakFocusHandle> {
        self.focus_restore_task = None;
        self.pending_focus_restore
            .take()
            .or_else(|| window.focused(cx).map(|h| h.downgrade()))
    }

    fn restore_overlay_focus(
        &mut self,
        previous: Option<WeakFocusHandle>,
        deferred: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_restore_task = None;
        self.pending_focus_restore = None;
        let Some(handle) = previous.and_then(|h| h.upgrade()) else {
            return;
        };
        if deferred {
            self.pending_focus_restore = Some(handle.downgrade());
            self.focus_restore_task = Some(cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(*ANIMATION_DURATION).await;
                let _ = this.update_in(cx, |this, window, cx| {
                    this.pending_focus_restore = None;
                    window.focus(&handle, cx);
                });
            }));
        } else {
            window.focus(&handle, cx);
        }
    }

    pub fn open_dialog_owned(
        &mut self,
        build: impl Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static,
        on_closed: impl Fn(OverlayCloseReason, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> OverlayToken {
        let previous_focused_handle = self.take_previous_focus(window, cx);
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        let selection_scope = self.allocate_text_selection_scope();
        let (lifetime, token) = self.overlay_lifetime(Rc::new(on_closed), cx);
        self.active_dialogs.push(ActiveDialog {
            lifetime,
            focus_handle,
            previous_focused_handle,
            selection_scope,
            builder: Rc::new(build),
        });
        gpui_base::TextSelection::clear(window, cx);
        cx.notify();
        token
    }

    pub fn open_alert_dialog_owned(
        &mut self,
        build: impl Fn(AlertDialog, &mut Window, &mut App) -> AlertDialog + 'static,
        on_closed: impl Fn(OverlayCloseReason, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> OverlayToken {
        self.open_dialog_owned(
            move |_, window, cx| build(AlertDialog::new(cx), window, cx).build_surface(window, cx),
            on_closed,
            window,
            cx,
        )
    }

    pub fn open_dialog<F>(&mut self, build: F, window: &mut Window, cx: &mut Context<Self>)
    where
        F: Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static,
    {
        self.open_dialog_owned(build, |_, _, _| {}, window, cx);
    }

    pub(crate) fn close_overlay(
        &mut self,
        id: u64,
        reason: OverlayCloseReason,
        deferred: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some(index) = self.active_dialogs.iter().position(|d| d.lifetime.id == id) {
            let dialog = self.active_dialogs.remove(index);
            if let Some(next) = self.active_dialogs.get_mut(index) {
                next.previous_focused_handle = dialog.previous_focused_handle;
            } else {
                self.focused_input = None;
                self.restore_overlay_focus(dialog.previous_focused_handle, deferred, window, cx);
            }
            dialog.lifetime.retire(reason, window, cx);
        } else if self
            .active_sheet
            .as_ref()
            .is_some_and(|s| s.lifetime.id == id)
        {
            let sheet = self.active_sheet.take().unwrap();
            self.sheet_size = None;
            if let Some(first) = self.active_dialogs.first_mut() {
                first.previous_focused_handle = sheet.previous_focused_handle;
            } else {
                self.focused_input = None;
                self.restore_overlay_focus(sheet.previous_focused_handle, deferred, window, cx);
            }
            sheet.lifetime.retire(reason, window, cx);
        } else {
            return false;
        }
        gpui_base::TextSelection::clear(window, cx);
        cx.notify();
        true
    }

    pub fn close_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.active_dialogs.last().map(|d| d.lifetime.id) {
            self.close_overlay(id, OverlayCloseReason::Programmatic, false, window, cx);
        }
    }
    pub fn close_all_dialogs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        while !self.active_dialogs.is_empty() {
            self.close_dialog(window, cx);
        }
    }

    pub fn open_sheet_owned(
        &mut self,
        placement: Placement,
        build: impl Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static,
        on_closed: impl Fn(OverlayCloseReason, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> OverlayToken {
        let current_focus = if self.active_dialogs.is_empty() {
            self.take_previous_focus(window, cx)
        } else {
            // A sheet cannot supersede the deferred focus of a surviving dialog.
            window.focused(cx).map(|h| h.downgrade())
        };
        let previous_focused_handle = if let Some(old) = self.active_sheet.take() {
            old.lifetime
                .retire(OverlayCloseReason::Replaced, window, cx);
            old.previous_focused_handle
        } else if let Some(dialog) = self.active_dialogs.first() {
            dialog.previous_focused_handle.clone()
        } else {
            current_focus
        };
        let focus_handle = cx.focus_handle();
        if let Some(dialog) = self.active_dialogs.first_mut() {
            dialog.previous_focused_handle = Some(focus_handle.downgrade());
            // A sheet is below every dialog. Do not move focus behind the stack.
        } else {
            focus_handle.focus(window, cx);
        }
        let selection_scope = self.allocate_text_selection_scope();
        let (lifetime, token) = self.overlay_lifetime(Rc::new(on_closed), cx);
        self.active_sheet = Some(ActiveSheet {
            lifetime,
            focus_handle,
            previous_focused_handle,
            placement,
            selection_scope,
            builder: Rc::new(build),
        });
        self.sheet_size = None;
        gpui_base::TextSelection::clear(window, cx);
        cx.notify();
        token
    }

    pub fn open_sheet_at<F>(
        &mut self,
        placement: Placement,
        build: F,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        F: Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static,
    {
        self.open_sheet_owned(placement, build, |_, _, _| {}, window, cx);
    }
    pub fn close_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.active_sheet.as_ref().map(|s| s.lifetime.id) {
            self.close_overlay(id, OverlayCloseReason::Programmatic, false, window, cx);
        }
    }
}
