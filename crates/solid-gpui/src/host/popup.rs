use super::*;
use gpui::{
    Bounds, FocusHandle, Pixels, Point,
    popup::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupOptions},
};

#[derive(Clone, Copy, PartialEq)]
struct PopupGeometry {
    anchor: Bounds<Pixels>,
    owner_bounds: Bounds<Pixels>,
    scale: f32,
    display: Option<(gpui::DisplayId, Bounds<Pixels>, Bounds<Pixels>)>,
}

pub(super) struct PopupSession {
    owner: u32,
    epoch: u32,
    anchor_node: u32,
    request: Option<CommandMeta>,
    open_request: u32,
    activated: bool,
    size: gpui::Size<Pixels>,
    placement: u32,
    gap: f32,
    geometry: Option<PopupGeometry>,
    return_focus: Option<FocusHandle>,
}

impl NativeStateRegistry {
    pub(super) fn open_popup(
        &mut self,
        command: Command,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let meta = command.meta;
        let CommandOperation::OpenPopup {
            anchor_node_id,
            width,
            height,
            placement,
            gap,
        } = command.operation
        else {
            unreachable!()
        };
        if self.popups.len() >= 32 {
            self.send_command_result(
                meta,
                CommandKind::OpenPopup,
                false,
                Some("at most 32 system popovers may be open".into()),
                None,
                cx,
            );
            return Ok(());
        }
        let owner = &self.surfaces[&meta.surface_id];
        if !owner
            .root
            .read(cx)
            .store()
            .get(anchor_node_id)
            .is_some_and(|node| {
                matches!(
                    node.kind,
                    crate::tree::KIND_VIEW | crate::tree::KIND_PRESSABLE
                )
            })
        {
            self.send_command_result(
                meta,
                CommandKind::OpenPopup,
                false,
                Some("popup anchor must be a mounted View or Pressable".into()),
                None,
                cx,
            );
            return Ok(());
        }
        let root = owner.root.clone();
        let return_focus = owner
            .window
            .update(cx, |_, window, cx| window.focused(cx))
            .ok()
            .flatten();
        let id = self.allocate_surface_id()?;
        self.popups.insert(
            id,
            PopupSession {
                owner: meta.surface_id,
                epoch: meta.epoch,
                anchor_node: anchor_node_id,
                request: Some(meta),
                open_request: meta.request_id,
                activated: false,
                size: size(px(width as f32), px(height as f32)),
                placement,
                gap,
                geometry: None,
                return_focus,
            },
        );
        let registry = cx.weak_entity();
        let owner_id = meta.surface_id;
        root.update(cx, |root, cx| {
            root.popup_anchors.insert(anchor_node_id);
            root.popup_observer = Some(Rc::new(move |cx| {
                let _ = registry.update(cx, |registry, cx| registry.reconcile_popups(owner_id, cx));
            }));
            cx.notify();
        });
        self.refresh_popup_input(owner_id, cx);
        Ok(())
    }

    pub(super) fn reconcile_popups(&mut self, owner_id: u32, cx: &mut Context<Self>) {
        let ids: Vec<_> = self
            .popups
            .iter()
            .filter(|(_, p)| p.owner == owner_id)
            .map(|(&id, _)| id)
            .collect();
        for id in ids {
            let Some(session) = self.popups.get(&id) else {
                continue;
            };
            let Some(owner) = self.surfaces.get(&owner_id) else {
                self.close_popup(id, false, cx);
                continue;
            };
            let root = owner.root.read(cx);
            let anchor = (root.store().epoch() == session.epoch)
                .then(|| root.popup_anchor(session.anchor_node))
                .flatten();
            let Some(anchor) = anchor else {
                self.fail_popup(id, "popup anchor is no longer visible", cx);
                continue;
            };
            let Ok(geometry) = owner.window.update(cx, |_, window, cx| PopupGeometry {
                anchor,
                owner_bounds: window.bounds(),
                scale: window.scale_factor(),
                display: window
                    .display(cx)
                    .map(|display| (display.id(), display.bounds(), display.visible_bounds())),
            }) else {
                self.fail_popup(id, "popup owner is unavailable", cx);
                continue;
            };
            if session.geometry == Some(geometry) {
                continue;
            }
            let parent = owner.window;
            if self.surfaces.contains_key(&id) {
                let window = self.surfaces[&id].window;
                let result = window.update(cx, |_, window, _| window.reposition_popup(anchor));
                match result {
                    Ok(Ok(())) => {
                        self.popups.get_mut(&id).unwrap().geometry = Some(geometry);
                    }
                    _ => self.fail_popup(id, "native popup reposition failed", cx),
                }
                continue;
            }
            let (popup_anchor, gravity, offset) = placement(session.placement, session.gap);
            let options = WindowOptions {
                titlebar: None,
                window_background: gpui::WindowBackgroundAppearance::Transparent,
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    Point::default(),
                    session.size,
                ))),
                kind: WindowKind::AnchoredPopup(PopupOptions {
                    parent,
                    anchor_rect: anchor,
                    anchor: popup_anchor,
                    gravity,
                    offset,
                    constraint_adjustment: PopupConstraintAdjustment::all(),
                    grab: false,
                }),
                is_movable: false,
                is_resizable: false,
                is_minimizable: false,
                focus: false,
                show: false,
                ..WindowOptions::default()
            };
            let opened = self.profile.open_window(
                options,
                self.runtime.clone(),
                self.profile.extension_registry(),
                cx,
            );
            match opened {
                Ok((window, root)) => {
                    let registry = cx.weak_entity();
                    let activation = window.update(cx, |_, window, cx| {
                        root.update(cx, |_, cx| {
                            cx.observe_window_activation(window, move |_, window, cx| {
                                if window.is_window_active() {
                                    let _ = registry.update(cx, |registry, _| {
                                        if let Some(popup) = registry.popups.get_mut(&id) {
                                            popup.activated = true;
                                        }
                                    });
                                }
                                let registry = registry.clone();
                                cx.defer(move |cx| {
                                    let _ = registry.update(cx, |registry, cx| {
                                        registry.dismiss_unrelated_popups(cx)
                                    });
                                });
                            })
                        })
                    });
                    let Ok(activation) = activation else {
                        self.fail_popup(id, "popup window was closed during creation", cx);
                        continue;
                    };
                    self.insert_surface(
                        id,
                        Surface {
                            window,
                            root,
                            _activation: activation,
                            close_policy: ClosePolicy::Allow,
                            pending_close_request: None,
                            next_close_request: 1,
                        },
                    );
                    self.refresh_popup_input(id, cx);
                    let session = self.popups.get_mut(&id).unwrap();
                    session.geometry = Some(geometry);
                    let request = session.request.take().unwrap();
                    self.send_command_result(
                        request,
                        CommandKind::OpenPopup,
                        true,
                        None,
                        Some(CommandValue::Number(id)),
                        cx,
                    );
                }
                Err(error) => self.fail_popup(id, &error, cx),
            }
        }
    }

    fn fail_popup(&mut self, id: u32, error: &str, cx: &mut Context<Self>) {
        if let Some(request) = self.popups.get_mut(&id).and_then(|p| p.request.take()) {
            self.send_command_result(
                request,
                CommandKind::OpenPopup,
                false,
                Some(error.into()),
                None,
                cx,
            );
        }
        self.close_popup(id, false, cx);
    }

    pub(super) fn close_popup_command(
        &mut self,
        command: Command,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let CommandOperation::ClosePopup { request_id } = command.operation else {
            unreachable!()
        };
        let id = self
            .popups
            .iter()
            .find(|(_, p)| {
                p.owner == command.meta.surface_id
                    && p.epoch == command.meta.epoch
                    && p.open_request == request_id
            })
            .map(|(&id, _)| id);
        self.send_command_result(command.meta, CommandKind::ClosePopup, true, None, None, cx);
        if let Some(id) = id {
            if let Some(request) = self.popups.get_mut(&id).and_then(|p| p.request.take()) {
                self.send_command_result(
                    request,
                    CommandKind::OpenPopup,
                    false,
                    Some("popup creation was cancelled".into()),
                    None,
                    cx,
                );
            }
            self.close_popup(id, true, cx);
        }
        Ok(())
    }

    pub(super) fn release_popup(&mut self, id: u32, cx: &mut Context<Self>) {
        let Some(popup) = self.popups.remove(&id) else {
            return;
        };
        if let Some(owner) = self.surfaces.get(&popup.owner) {
            let anchors = self
                .popups
                .values()
                .filter(|p| p.owner == popup.owner)
                .map(|p| p.anchor_node)
                .collect::<HashSet<_>>();
            owner.root.update(cx, |root, cx| {
                root.popup_anchors = anchors;
                if root.popup_anchors.is_empty() {
                    root.popup_observer = None;
                }
                cx.notify();
            });
        }
        self.refresh_popup_input(popup.owner, cx);
    }

    fn refresh_popup_input(&self, surface_id: u32, cx: &mut Context<Self>) {
        let Some(surface) = self.surfaces.get(&surface_id) else {
            return;
        };
        let listens = self.popups.contains_key(&surface_id)
            || self.popups.values().any(|p| p.owner == surface_id);
        let registry = cx.weak_entity();
        surface.root.update(cx, |root, cx| {
            root.popup_input = listens.then(|| {
                Rc::new(move |position: Option<Point<Pixels>>, cx: &mut App| {
                    let _ = registry.update(cx, |registry, cx| {
                        if let Some(position) = position {
                            let ids: Vec<_> = registry
                                .popups
                                .iter()
                                .filter(|(_, p)| {
                                    p.owner == surface_id
                                        && p.geometry.is_some_and(|geometry| {
                                            !geometry.anchor.contains(&position)
                                        })
                                })
                                .map(|(&id, _)| id)
                                .collect();
                            for id in ids {
                                registry.close_popup(id, false, cx);
                            }
                        } else {
                            let id = registry
                                .popups
                                .iter()
                                .filter(|(_, p)| p.owner == surface_id)
                                .map(|(&id, _)| id)
                                .max()
                                .or_else(|| {
                                    registry
                                        .popups
                                        .contains_key(&surface_id)
                                        .then_some(surface_id)
                                });
                            if let Some(id) = id {
                                registry.close_popup(id, true, cx);
                            }
                        }
                    });
                }) as Rc<dyn Fn(Option<Point<Pixels>>, &mut App)>
            });
            cx.notify();
        });
    }

    pub(super) fn popup_received_snapshot(&self, surface_id: u32, cx: &mut Context<Self>) {
        if self.popups.contains_key(&surface_id) {
            if let Some(surface) = self.surfaces.get(&surface_id) {
                let _ = surface.window.update(cx, |_, window, _| {
                    // Hidden X11 windows do not receive frame ticks. Admit mapping
                    // only once the initial content Snapshot has been installed.
                    window.activate_window();
                    window.on_next_frame(|window, _| {
                        window.on_next_frame(|window, cx| {
                            if window.is_window_active() {
                                window.focus_next(cx);
                            }
                        });
                    });
                });
            }
        }
    }

    fn close_popup(&mut self, id: u32, restore_focus: bool, cx: &mut Context<Self>) {
        self.close_owned_popups(id, cx);
        let focus = self.popups.get(&id).and_then(|p| {
            self.surfaces
                .get(&p.owner)
                .map(|s| (s.window, p.return_focus.clone()))
        });
        let window = self.surfaces.get(&id).map(|s| s.window);
        let was_active = window.is_some_and(|w| cx.active_window() == Some(w));
        self.release_popup(id, cx);
        self.retired_surface_ids.insert(id);
        if let Some(window) = window {
            cx.defer(move |cx| {
                let restore_focus =
                    restore_focus && was_active && cx.active_window() == Some(window);
                let _ = window.update(cx, |_, window, _| window.remove_window());
                if restore_focus {
                    if let Some((owner, focus)) = focus {
                        let _ = owner.update(cx, |_, window, cx| {
                            window.activate_window();
                            if let Some(focus) = focus {
                                focus.focus(window, cx);
                            }
                        });
                    }
                }
            });
        }
    }

    pub(super) fn close_owned_popups(&mut self, owner: u32, cx: &mut Context<Self>) {
        let mut ids: Vec<_> = self
            .popups
            .iter()
            .filter(|(_, p)| p.owner == owner)
            .map(|(&id, _)| id)
            .collect();
        ids.sort_unstable_by(|a, b| b.cmp(a));
        for id in ids {
            self.fail_popup(id, "popup owner was closed", cx);
        }
    }

    pub(super) fn dismiss_unrelated_popups(&mut self, cx: &mut Context<Self>) {
        let active = cx
            .active_window()
            .and_then(|window| self.windows.get(&window.window_id()).copied());
        // Read the platform's key window once. GPUI activation observers may
        // still describe the previous window during a native focus transfer.
        let mut family = HashSet::new();
        if let Some(mut current) = active {
            family.insert(current);
            while let Some(popup) = self.popups.get(&current) {
                current = popup.owner;
                family.insert(current);
            }
        }
        let mut ids: Vec<_> = self
            .popups
            .iter()
            .filter(|(id, p)| p.activated && !family.contains(id))
            .map(|(&id, _)| id)
            .collect();
        ids.sort_unstable_by(|a, b| b.cmp(a));
        for id in ids {
            self.close_popup(id, false, cx);
        }
        self.restore_keybindings(cx);
    }
}

fn placement(value: u32, gap: f32) -> (PopupAnchor, PopupGravity, Point<Pixels>) {
    use PopupAnchor as A;
    use PopupGravity as G;
    let (anchor, gravity, x, y) = match value {
        0 => (A::BottomLeft, G::BottomRight, 0.0, gap),
        1 => (A::Bottom, G::Bottom, 0.0, gap),
        2 => (A::BottomRight, G::BottomLeft, 0.0, gap),
        3 => (A::TopLeft, G::TopRight, 0.0, -gap),
        4 => (A::Top, G::Top, 0.0, -gap),
        5 => (A::TopRight, G::TopLeft, 0.0, -gap),
        6 => (A::Left, G::Left, -gap, 0.0),
        7 => (A::TopLeft, G::BottomLeft, -gap, 0.0),
        8 => (A::BottomLeft, G::TopLeft, -gap, 0.0),
        9 => (A::Right, G::Right, gap, 0.0),
        10 => (A::TopRight, G::BottomRight, gap, 0.0),
        _ => (A::BottomRight, G::TopRight, gap, 0.0),
    };
    (anchor, gravity, gpui::point(px(x), px(y)))
}
