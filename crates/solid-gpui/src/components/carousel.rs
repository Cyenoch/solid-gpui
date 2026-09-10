//! One retained native viewport; logical selection follows Host Node identity on reorder.
use super::{ControlSize, primitives::Orientation};
use crate::native::{
    ComponentDefinition, ControlledBinding, ElementContext, Event, NativeChildren, NativeView,
    ViewCommand,
};
use gpui::{
    AnyElement, AppContext, Context, ElementId, Entity, IntoElement, ParentElement, Render,
    StyleRefinement, Styled, Subscription, Window, prelude::FluentBuilder, px,
};
use gpui_component::Sizable;
use gpui_component::carousel::{
    Carousel as NativeCarousel, CarouselContent, CarouselEvent, CarouselItem, CarouselNext,
    CarouselPagination, CarouselPaginationItem, CarouselPrevious, CarouselState,
};

#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CarouselItemProps {
    pub accessibility_label: Option<String>,
}

struct Slide {
    id: ElementId,
    label: Option<String>,
    style: StyleRefinement,
    children: Vec<AnyElement>,
}
impl Styled for Slide {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
fn slide(props: &CarouselItemProps, cx: &mut ElementContext) -> Slide {
    Slide {
        id: cx.id(),
        label: props.accessibility_label.clone(),
        style: StyleRefinement::default(),
        children: cx.children().collect(),
    }
}

#[crate::native_type]
#[derive(Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CarouselProps {
    pub orientation: Orientation,
    pub selected_index: Option<usize>,
    pub default_selected_index: usize,
    pub ack_edit_seq: Option<u32>,
    pub looping: bool,
    pub controls: bool,
    pub pagination: bool,
    pub size: ControlSize,
    pub gap: f32,
    pub viewport_height: Option<f32>,
    pub accessibility_label: String,
}
impl Default for CarouselProps {
    fn default() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            selected_index: None,
            default_selected_index: 0,
            ack_edit_seq: None,
            looping: false,
            controls: true,
            pagination: false,
            size: ControlSize::Medium,
            gap: 16.,
            viewport_height: None,
            accessibility_label: "Carousel".into(),
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct CarouselChange {
    pub index: usize,
    pub edit_seq: u32,
}

struct Carousel {
    props: CarouselProps,
    children: NativeChildren,
    keys: std::rc::Rc<Vec<u32>>,
    state: Entity<CarouselState>,
    sequence: u32,
    _subscription: Subscription,
}
impl NativeView for Carousel {
    type Props = CarouselProps;
    type Event = CarouselChange;
    fn accepts_children() -> bool {
        true
    }
    fn slots() -> &'static [&'static str] {
        &["previous", "next"]
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn controlled() -> Option<ControlledBinding> {
        Some(ControlledBinding {
            value_prop: "selectedIndex",
            event_id: 1,
            sequence_field: "editSeq",
            ack_prop: "ackEditSeq",
        })
    }
    fn validate_props(props: &Self::Props) -> Result<(), String> {
        if !props.gap.is_finite() || !(0.0..=4096.0).contains(&props.gap) {
            return Err("carousel gap must be between 0 and 4096".into());
        }
        if props
            .viewport_height
            .is_some_and(|v| !v.is_finite() || !(1.0..=100000.0).contains(&v))
            || props.orientation == Orientation::Vertical && props.viewport_height.is_none()
        {
            return Err("vertical carousels require viewportHeight between 1 and 100000".into());
        }
        Ok(())
    }
    fn validate_children(
        props: &Self::Props,
        children: &crate::ExtensionChildSummary,
    ) -> Result<(), String> {
        if children.count > 1024 {
            return Err("a carousel supports at most 1024 items".into());
        }
        if props
            .selected_index
            .is_some_and(|index| index >= children.count.max(1))
        {
            return Err("selectedIndex must identify a committed carousel item".into());
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
        let keys = children.content().node_ids();
        let state = cx.new(|_| {
            CarouselState::new(keys.len())
                .with_axis(props.orientation.into())
                .with_looping(props.looping)
                .with_selected_index(props.selected_index.unwrap_or(props.default_selected_index))
        });
        let subscription = cx.subscribe(&state, move |this, _, event_value, _| {
            let CarouselEvent::Change(index) = event_value;
            this.sequence = this
                .sequence
                .checked_add(1)
                .expect("carousel sequence exhausted");
            event.emit(CarouselChange {
                index: *index,
                edit_seq: this.sequence,
            });
        });
        Self {
            props,
            children,
            keys,
            state,
            sequence: 0,
            _subscription: subscription,
        }
    }
    fn update(&mut self, props: Self::Props, _: &mut Window, cx: &mut Context<Self>) {
        let keys = self.children.content().node_ids();
        let selected = self.state.read(cx).selected_index();
        let retained = selected
            .and_then(|index| self.keys.get(index))
            .and_then(|key| keys.iter().position(|candidate| candidate == key));
        let acknowledged = props.ack_edit_seq.is_none_or(|ack| ack >= self.sequence);
        self.state.update(cx, |state, cx| {
            if keys != self.keys {
                state.reconcile_items(keys.len(), retained.or(selected), cx);
            }
            state.set_axis(props.orientation.into(), cx);
            state.set_looping(props.looping, cx);
            if acknowledged
                && let Some(index) = props.selected_index
                && state.selected_index() != Some(index)
            {
                state.set_selected_index(index, cx);
            }
        });
        self.keys = keys;
        self.props = props;
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("getSelectedIndex", |this, (): (), _, cx| {
                Ok(this.state.read(cx).selected_index())
            }),
            ViewCommand::new("select", |this, index: usize, _, cx| {
                if index >= this.state.read(cx).item_count() {
                    return Err("carousel index outside committed items".into());
                }
                this.state.update(cx, |state, cx| {
                    state.select_index(index, cx);
                });
                Ok(())
            }),
            ViewCommand::new("next", |this, (): (), _, cx| {
                Ok(this.state.update(cx, |state, cx| state.select_next(cx)))
            }),
            ViewCommand::new("previous", |this, (): (), _, cx| {
                Ok(this.state.update(cx, |state, cx| state.select_previous(cx)))
            }),
        ]
    }
}
impl Render for Carousel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let horizontal = self.props.orientation == Orientation::Horizontal;
        let gap = px(self.props.gap);
        let track = if horizontal {
            StyleRefinement::default().ml(-gap)
        } else {
            StyleRefinement::default().mt(-gap)
        };
        let items = self
            .children
            .typed_children::<Slide>(cx)
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                child.map_native(|slide| {
                    let mut item = CarouselItem::new(slide.id, index, &self.state)
                        .when(horizontal, |item| item.pl(gap))
                        .when(!horizontal, |item| item.pt(gap))
                        .children(slide.children);
                    if let Some(label) = slide.label {
                        item = item.accessibility_label(label);
                    }
                    gpui::Refineable::refine(item.style(), &slide.style);
                    item
                })
            });
        let content = CarouselContent::new(&self.state)
            .w_full()
            .track_style(track)
            .when_some(self.props.viewport_height, |content, height| {
                content.h(px(height))
            })
            .children(items);
        let previous = self.children.slot("previous");
        let next = self.children.slot("next");
        NativeCarousel::new("carousel", &self.state)
            .w_full()
            .accessibility_label(self.props.accessibility_label.clone())
            .child(content)
            .when(self.props.controls, |carousel| {
                carousel
                    .child(
                        CarouselPrevious::new(&self.state)
                            .with_size(self.props.size)
                            .when(!previous.node_ids().is_empty(), |control| {
                                control.child(previous)
                            }),
                    )
                    .child(
                        CarouselNext::new(&self.state)
                            .with_size(self.props.size)
                            .when(!next.node_ids().is_empty(), |control| control.child(next)),
                    )
            })
            .when(self.props.pagination, |carousel| {
                carousel.child(CarouselPagination::new().children(
                    self.keys.iter().enumerate().map(|(index, key)| {
                        CarouselPaginationItem::new(
                            ("carousel-page", *key as u64),
                            index,
                            &self.state,
                        )
                    }),
                ))
            })
    }
}

pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::styled_descriptor::<CarouselItemProps, _>(
            "CarouselItem",
            vec![],
            slide,
        )
        .with_contract(include_str!("carousel.rs")),
        ComponentDefinition::view::<Carousel>("Carousel")
            .with_child_type::<Slide>()
            .with_contract(include_str!("carousel.rs")),
    ]
}
