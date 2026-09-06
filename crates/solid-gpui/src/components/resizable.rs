use super::primitives::Orientation;
use crate::native::{
    ComponentDefinition, ControlledBinding, Deserialize, ElementContext, Event, NativeChildren,
    NativeView, Serialize, TS, ViewCommand,
};
use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Render, Window, px};

#[derive(Clone, Debug, Serialize, TS)]
pub struct PanelLimits {
    pub min: f32,
    pub max: f32,
}
impl Default for PanelLimits {
    fn default() -> Self {
        Self {
            min: 100.,
            max: 1_000_000.,
        }
    }
}
impl<'de> Deserialize<'de> for PanelLimits {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            min: f32,
            max: f32,
        }
        let v = Wire::deserialize(d)?;
        if !v.min.is_finite()
            || !v.max.is_finite()
            || v.min < 0.
            || v.min > v.max
            || v.max > 1_000_000.
        {
            return Err(serde::de::Error::custom(
                "panel limits require 0 <= min <= max <= 1000000",
            ));
        }
        Ok(Self {
            min: v.min,
            max: v.max,
        })
    }
}
#[derive(Clone, Debug, Serialize, TS)]
pub struct PanelSize(pub f32);
impl<'de> Deserialize<'de> for PanelSize {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = f32::deserialize(d)?;
        if !v.is_finite() || !(0.0..=1_000_000.0).contains(&v) {
            return Err(serde::de::Error::custom(
                "panel size must be between 0 and 1000000",
            ));
        }
        Ok(Self(v))
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct ResizablePanelProps {
    pub size: Option<PanelSize>,
    pub limits: PanelLimits,
    pub visible: bool,
}
impl Default for ResizablePanelProps {
    fn default() -> Self {
        Self {
            size: None,
            limits: PanelLimits::default(),
            visible: true,
        }
    }
}
fn panel(props: &ResizablePanelProps, cx: &mut ElementContext) -> gpui_component::ResizablePanel {
    let mut panel = gpui_component::resizable_panel(cx.id())
        .size_range(px(props.limits.min)..px(props.limits.max))
        .visible(props.visible);
    if let Some(size) = &props.size {
        panel = panel.size(px(size.0));
    }
    panel.children(cx.children())
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ResizablePanelGroupProps {
    pub orientation: Orientation,
    pub sizes: Option<Vec<f32>>,
    pub ack_edit_seq: Option<u32>,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct PanelResize {
    pub sizes: Vec<f32>,
    pub edit_seq: u32,
}
#[crate::native_type]
#[derive(Clone)]
pub struct ResizePanel {
    pub index: usize,
    pub size: f32,
}
pub struct ResizablePanelGroup {
    props: ResizablePanelGroupProps,
    children: NativeChildren,
    state: Entity<gpui_component::ResizableState>,
    seq: u32,
    _subscription: gpui::Subscription,
}
impl ResizablePanelGroup {
    fn sync(
        state: &mut gpui_component::ResizableState,
        props: &ResizablePanelGroupProps,
        children: &NativeChildren,
        cx: &mut Context<gpui_component::ResizableState>,
    ) {
        let keys = children
            .content()
            .node_ids()
            .iter()
            .map(|id| gpui::ElementId::Integer(*id as u64))
            .collect::<Vec<_>>();
        state.sync_panels(props.orientation.into(), &keys, cx);
    }
}
impl NativeView for ResizablePanelGroup {
    type Props = ResizablePanelGroupProps;
    type Event = PanelResize;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "resize"
    }
    fn controlled() -> Option<ControlledBinding> {
        Some(ControlledBinding {
            value_prop: "sizes",
            event_id: 1,
            sequence_field: "editSeq",
            ack_prop: "ackEditSeq",
        })
    }
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        if p.sizes.as_ref().is_some_and(|sizes| {
            sizes.len() > 128
                || sizes
                    .iter()
                    .any(|v| !v.is_finite() || *v < 0. || *v > 1_000_000.)
        }) {
            return Err(
                "sizes must contain up to 128 finite panel extents between 0 and 1000000".into(),
            );
        }
        Ok(())
    }
    fn validate_children(p: &Self::Props, c: &crate::ExtensionChildSummary) -> Result<(), String> {
        if c.count > 128 || p.sizes.as_ref().is_some_and(|s| s.len() != c.count) {
            return Err("sizes must contain one extent per panel, with at most 128 panels".into());
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|_| gpui_component::ResizableState::default());
        state.update(cx, |s, cx| {
            Self::sync(s, &props, &children, cx);
            if let Some(sizes) = &props.sizes {
                s.adopt_sizes(&sizes.iter().map(|v| Some(px(*v))).collect::<Vec<_>>(), cx);
            }
        });
        let subscription = cx.subscribe(&state, move |this, state, _, cx| {
            this.seq = this.seq.checked_add(1).expect("resize sequence exhausted");
            event.emit(PanelResize {
                sizes: state.read(cx).sizes().iter().map(|v| v.as_f32()).collect(),
                edit_seq: this.seq,
            });
        });
        Self {
            props,
            children,
            state,
            seq: 0,
            _subscription: subscription,
        }
    }
    fn update(&mut self, p: Self::Props, _: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            Self::sync(state, &p, &self.children, cx);
            let acknowledged = p.ack_edit_seq.is_none_or(|ack| ack >= self.seq);
            if acknowledged
                && (p.sizes != self.props.sizes || p.ack_edit_seq != self.props.ack_edit_seq)
                && let Some(sizes) = &p.sizes
            {
                state.adopt_sizes(&sizes.iter().map(|v| Some(px(*v))).collect::<Vec<_>>(), cx);
            }
        });
        self.props = p;
        cx.notify();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("getSizes", |this, (): (), _, cx| {
                Ok(this
                    .state
                    .read(cx)
                    .keys()
                    .iter()
                    .map(|key| {
                        this.state
                            .read(cx)
                            .measured_size_for_key(key)
                            .map(|s| s.as_f32())
                    })
                    .collect::<Vec<_>>())
            }),
            ViewCommand::new("resizePanel", |this, p: ResizePanel, window, cx| {
                if !p.size.is_finite() || p.size < 0. || p.size > 1_000_000. {
                    return Err("invalid panel size".into());
                }
                let state = this.state.read(cx);
                let Some(key) = state.keys().get(p.index) else {
                    return Err("panel index outside committed children".into());
                };
                if state.measured_size_for_key(key).is_none() || state.container_size() <= px(0.) {
                    return Err("panel layout is not yet available".into());
                }
                this.state
                    .update(cx, |s, cx| s.resize_panel(p.index, px(p.size), window, cx));
                Ok(())
            }),
        ]
    }
}
impl Render for ResizablePanelGroup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui_component::ResizablePanelGroup::new("resizable-group")
            .axis(self.props.orientation.into())
            .with_state(&self.state)
            .children(
                self.children
                    .typed_children::<gpui_component::ResizablePanel>(cx),
            )
    }
}
pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::styled_element::<ResizablePanelProps, _>(
            "ResizablePanel",
            vec![],
            panel,
        )
        .with_typed_parent_required()
        .with_contract(include_str!("resizable.rs")),
        ComponentDefinition::view::<ResizablePanelGroup>("ResizablePanelGroup")
            .with_child_type::<gpui_component::ResizablePanel>()
            .with_contract(include_str!("resizable.rs")),
    ]
}
