//! The upstream input group: one native frame around a retained text control
//! and its explicitly aligned addon rows.
//!
//! Ownership follows the existing retained-control seams: the control slot
//! child stays the retained `Input`/`Textarea` view and keeps every capability
//! (controlled value, tokens, paste, prefix/suffix, commands); the group owns
//! only the upstream frame and composition, restyling the control exactly as
//! upstream `render_control` does. The group also overlays its
//! disabled/readonly policy onto the control's retained engine, releasing it
//! on unmount or when the control moves away.
use super::input::{MultiLine, SingleLine, TextControl};
use super::{ControlSize, icon_source::ComponentIcon};
use crate::native::{
    ComponentDefinition, ElementContext, Event, EventDefinition, NativeChildren, NativeView,
};
use gpui::{Context, Entity, IntoElement, ParentElement, Render, Window};
use gpui_component::input::{
    InputGroup as GpuiInputGroup, InputGroupAddon, InputGroupAddonAlignment,
    InputGroupButton as GpuiInputGroupButton, InputGroupText as GpuiInputGroupText,
};
use gpui_component::{Disableable, FocusableExt, Selectable, Sizable};
use std::marker::PhantomData;

#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct InputGroupProps {
    pub size: ControlSize,
    pub disabled: bool,
    pub readonly: bool,
    pub invalid: bool,
    pub focus_ring: bool,
    pub aria_label: Option<String>,
}
impl Default for InputGroupProps {
    fn default() -> Self {
        Self {
            size: ControlSize::default(),
            disabled: false,
            readonly: false,
            invalid: false,
            focus_ring: true,
            aria_label: None,
        }
    }
}

/// One registered input group frame around its control slot child. The frame
/// renders upstream; the retained control keeps its entity as renderer of
/// record, so the group re-renders whenever the control updates and while its
/// engine reports focus or content changes.
struct InputGroup<M: super::input::TextMode> {
    owner: gpui::EntityId,
    props: InputGroupProps,
    children: NativeChildren,
    observed: Option<gpui::EntityId>,
    attached: Option<Entity<TextControl<M>>>,
    synced: Option<(gpui::EntityId, bool, bool)>,
    _observation: Option<gpui::Subscription>,
    _state_subscription: Option<gpui::Subscription>,
    _mode: PhantomData<M>,
}

impl<M: super::input::TextMode> InputGroup<M> {
    fn control(&self, cx: &gpui::App) -> crate::native::NativeChild<Entity<TextControl<M>>> {
        let children = self
            .children
            .slot("control")
            .native_items::<Entity<TextControl<M>>>(cx);
        let Some(child) = children.into_iter().next() else {
            panic!("validated input group control slot");
        };
        child
    }

    fn refresh_control(
        &mut self,
        control: &Entity<TextControl<M>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let owner = self.owner;
        let entity_id = control.entity_id();
        if let Some(attached) = self
            .attached
            .as_ref()
            .filter(|attached| attached.entity_id() != entity_id)
        {
            // The control moved away from this group; release this group's
            // overlay so only the control's own values remain.
            attached.update(cx, |control, cx| control.clear_group_editing(owner, cx));
            self.attached = None;
            self.observed = None;
        }
        let fresh = self.synced.is_none_or(|(id, disabled, readonly)| {
            id != entity_id || disabled != self.props.disabled || readonly != self.props.readonly
        });
        if fresh {
            // The frame's editing policy applies to the retained engine, not
            // just the frame; lifting it restores exactly the control's own
            // values.
            control.update(cx, |control, cx| {
                control.apply_group_editing(owner, self.props.disabled, self.props.readonly, cx)
            });
            self.synced = Some((entity_id, self.props.disabled, self.props.readonly));
        }
        if self.observed != Some(entity_id) {
            self._observation = Some(cx.observe(control, |_, _, cx| cx.notify()));
            let state = control.read(cx).state().clone();
            self._state_subscription =
                Some(
                    cx.subscribe_in(&state, window, |_, _, event, _, cx| match event {
                        gpui_component::input::InputEvent::Focus
                        | gpui_component::input::InputEvent::Blur
                        | gpui_component::input::InputEvent::Change => cx.notify(),
                        _ => {}
                    }),
                );
            self.attached = Some(control.clone());
            self.observed = Some(entity_id);
        }
    }
}

impl<M: super::input::TextMode> NativeView for InputGroup<M> {
    type Props = InputGroupProps;
    type Event = ();

    fn accepts_children() -> bool {
        true
    }
    fn slots() -> &'static [&'static str] {
        &["control", "start", "end", "top", "bottom"]
    }
    fn emits_primary_event() -> bool {
        false
    }
    fn validate_children(
        _: &Self::Props,
        children: &crate::ExtensionChildSummary,
    ) -> Result<(), String> {
        if children.content_count() > 0 {
            return Err("input group takes addon content only through its named slots".into());
        }
        let control = children
            .slot(1)
            .ok_or("input group requires a control slot group")?;
        if control.count != 1 {
            return Err("input group control slot must hold exactly one control".into());
        }
        // Type identity, not DTO shape: only the registered retained control
        // entity is accepted, so arbitrary components that happen to decode
        // as control props are rejected at publication.
        let expected = std::any::TypeId::of::<Entity<TextControl<M>>>();
        if control.element_types != vec![Some(expected)] {
            return Err(
                if <M::Mode as gpui_base::input::InputModeKind>::MULTI_LINE {
                    "InputGroupTextarea control slot must hold exactly one Textarea".to_owned()
                } else {
                    "InputGroup control slot must hold exactly one Input".to_owned()
                },
            );
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        _event: Event<()>,
        children: NativeChildren,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            owner: cx.entity_id(),
            props,
            children,
            observed: None,
            attached: None,
            synced: None,
            _observation: None,
            _state_subscription: None,
            _mode: PhantomData,
        }
    }
    fn update(&mut self, props: Self::Props, _window: &mut Window, _cx: &mut Context<Self>) {
        self.props = props;
    }
    fn unmount(&mut self, _window: &mut Window, cx: &mut gpui::App) {
        let owner = self.owner;
        if let Some(attached) = self.attached.take() {
            attached.update(cx, |control, cx| control.clear_group_editing(owner, cx));
        }
    }
}

impl<M: super::input::TextMode> Render for InputGroup<M> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let control = self.control(cx);
        let representation = control
            .native
            .read(cx)
            .group_representation()
            .expect("validated input group control");
        self.refresh_control(&control.native, window, cx);

        let entity_id = control.native.entity_id();
        let mut group = GpuiInputGroup::new(gpui::ElementId::View(entity_id))
            .input_with_boundary(representation, control.boundary.clone())
            .with_size(self.props.size)
            .disabled(self.props.disabled)
            .readonly(self.props.readonly)
            .invalid(self.props.invalid);
        if !self.props.focus_ring {
            group = group.focus_ring(false);
        }
        if let Some(label) = &self.props.aria_label {
            group = group.aria_label(label.clone());
        }
        for (slot, alignment) in [
            ("start", InputGroupAddonAlignment::InlineStart),
            ("end", InputGroupAddonAlignment::InlineEnd),
            ("top", InputGroupAddonAlignment::BlockStart),
            ("bottom", InputGroupAddonAlignment::BlockEnd),
        ] {
            let content = self.children.slot(slot);
            if content.is_empty() {
                continue;
            }
            // Addon children keep their host boundary until the upstream
            // addon has applied group presentation, so a direct button stays
            // scan-visible and inherits the group's disabled state.
            let mut addon = InputGroupAddon::new(slot).align(alignment);
            for child in content.native_parts(cx) {
                addon = addon.child_with_boundary(child.native, child.boundary);
            }
            group = group.addon(addon);
        }
        group.into_any_element()
    }
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct InputGroupTextProps {}

fn group_text(_: &InputGroupTextProps, cx: &mut ElementContext) -> GpuiInputGroupText {
    let mut text = GpuiInputGroupText::new();
    text.extend(cx.children());
    text
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct InputGroupButtonProps {
    pub label: Option<String>,
    pub icon: Option<ComponentIcon>,
    pub disabled: bool,
    pub selected: bool,
    pub outline: bool,
    pub size: ControlSize,
    pub tab_index: i32,
    pub dropdown_caret: bool,
    pub tooltip: Option<String>,
    pub accessibility_label: Option<String>,
    pub loading: bool,
    pub loading_icon: Option<ComponentIcon>,
}
impl Default for InputGroupButtonProps {
    fn default() -> Self {
        Self {
            label: None,
            icon: None,
            disabled: false,
            selected: false,
            outline: false,
            size: ControlSize::Xsmall,
            tab_index: 0,
            dropdown_caret: false,
            tooltip: None,
            accessibility_label: None,
            loading: false,
            loading_icon: None,
        }
    }
}

fn group_button(p: &InputGroupButtonProps, cx: &mut ElementContext) -> GpuiInputGroupButton {
    let mut button = GpuiInputGroupButton::new(cx.id())
        .with_size(p.size)
        .disabled(p.disabled)
        .selected(p.selected)
        .tab_index(p.tab_index as isize)
        .dropdown_caret(p.dropdown_caret)
        .loading(p.loading);
    if p.outline {
        button = button.outline();
    }
    if let Some(label) = &p.label {
        button = button.label(label.clone());
    }
    if let Some(icon) = &p.icon {
        button = button.icon(icon.native());
    }
    if let Some(icon) = &p.loading_icon {
        button = button.loading_icon(icon.native());
    }
    if let Some(label) = &p.accessibility_label {
        button = button.accessibility_label(label.clone());
    }
    if let Some(tooltip) = &p.tooltip {
        button = button.tooltip(tooltip.clone());
    }
    let press = cx.event::<()>("press");
    if press.is_subscribed() {
        button = button.on_click(move |_, _, _| press.emit(()));
    }
    button
}

/// Registration for the shared component module. Chain this after
/// `input::definitions()` in `components::native_module`.
pub(crate) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::view::<InputGroup<SingleLine>>("InputGroup"),
        ComponentDefinition::view::<InputGroup<MultiLine>>("InputGroupTextarea"),
        ComponentDefinition::styled_element::<InputGroupTextProps, GpuiInputGroupText>(
            "InputGroupText",
            vec![],
            group_text,
        ),
        ComponentDefinition::element::<InputGroupButtonProps, GpuiInputGroupButton>(
            "InputGroupButton",
            vec![EventDefinition::new::<()>("press")],
            group_button,
        ),
    ]
    .into_iter()
    .map(|definition| definition.with_contract(include_str!("input_group.rs")))
    .collect()
}
