use bitflags::bitflags;
use thiserror::Error;

use crate::{AnyWindowHandle, Bounds, Pixels, Point, Size, point, px, size};

/// Options for a parent-anchored popup window such as a menu, dropdown, context menu or tooltip.
///
/// A popup is placed relative to an anchor rectangle on its parent window rather than at an
/// absolute screen position. The platform resolves the final position, so this works both on
/// systems where the compositor owns window placement (Wayland) and on platforms with absolute
/// coordinates.
///
/// The popup's size comes from [`WindowOptions::window_bounds`](crate::WindowOptions), whose
/// origin is ignored. All coordinates are in logical pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopupOptions {
    /// The window the popup is anchored to.
    pub parent: AnyWindowHandle,

    /// The rectangle the popup is positioned relative to, in the parent window's coordinate
    /// space (the same space element bounds are in). For example, a dropdown menu uses the
    /// bounds of the button that opened it.
    pub anchor_rect: Bounds<Pixels>,

    /// Which point of [`Self::anchor_rect`] the popup is anchored to.
    pub anchor: PopupAnchor,

    /// The direction in which the popup extends away from the anchor point. A dropdown that
    /// drops below its button anchors to [`PopupAnchor::BottomLeft`] with a gravity of
    /// [`PopupGravity::BottomRight`] so it grows down and to the right.
    pub gravity: PopupGravity,

    /// How the platform may adjust the popup if the requested placement would put it off-screen.
    pub constraint_adjustment: PopupConstraintAdjustment,

    /// An additional offset applied to the popup after anchoring.
    pub offset: Point<Pixels>,

    /// Whether the popup should take an explicit input grab.
    ///
    /// Grabbing popups behave like menus: they take keyboard focus and are dismissed when the
    /// user clicks outside of them or presses a dismissing key. Use it for menus and comboboxes,
    /// not for tooltips or other passive popups.
    ///
    /// A grab must be requested while the triggering input is still active, in practice the
    /// press of the mouse button that opens the popup. Open grabbing popups from a mouse-down
    /// handler rather than a click handler, otherwise the grab is refused.
    ///
    /// Automatic dismissal only covers input aimed at other applications. A click elsewhere in
    /// your own application still reaches it as usual, so closing the popup in that case is up
    /// to you. Nested grabbing popups must be closed in the reverse order they were opened.
    pub grab: bool,
}

/// The point of the anchor rectangle that a popup is anchored to.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PopupAnchor {
    /// Anchor to the center of the anchor rectangle.
    #[default]
    Center,
    /// Anchor to the center of the top edge.
    Top,
    /// Anchor to the center of the bottom edge.
    Bottom,
    /// Anchor to the center of the left edge.
    Left,
    /// Anchor to the center of the right edge.
    Right,
    /// Anchor to the top-left corner.
    TopLeft,
    /// Anchor to the bottom-left corner.
    BottomLeft,
    /// Anchor to the top-right corner.
    TopRight,
    /// Anchor to the bottom-right corner.
    BottomRight,
}

/// The direction in which a popup extends away from its anchor point.
///
/// For instance, a gravity of [`PopupGravity::BottomRight`] places the popup below and to the
/// right of the anchor point.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PopupGravity {
    /// The popup is centered over the anchor point.
    #[default]
    Center,
    /// The popup extends upwards from the anchor point.
    Top,
    /// The popup extends downwards from the anchor point.
    Bottom,
    /// The popup extends to the left of the anchor point.
    Left,
    /// The popup extends to the right of the anchor point.
    Right,
    /// The popup extends up and to the left of the anchor point.
    TopLeft,
    /// The popup extends down and to the left of the anchor point.
    BottomLeft,
    /// The popup extends up and to the right of the anchor point.
    TopRight,
    /// The popup extends down and to the right of the anchor point.
    BottomRight,
}

bitflags! {
    /// How a popup may be adjusted by the platform if the requested placement would put it
    /// off-screen. If no flags are set, the popup is placed exactly as requested and may be
    /// clipped.
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct PopupConstraintAdjustment: u32 {
        /// The popup may be slid horizontally to stay on-screen.
        const SLIDE_X = 1;
        /// The popup may be slid vertically to stay on-screen.
        const SLIDE_Y = 2;
        /// The popup's anchor and gravity may be flipped horizontally to stay on-screen.
        const FLIP_X = 4;
        /// The popup's anchor and gravity may be flipped vertically to stay on-screen.
        const FLIP_Y = 8;
        /// The popup may be shrunk horizontally to stay on-screen.
        const RESIZE_X = 16;
        /// The popup may be shrunk vertically to stay on-screen.
        const RESIZE_Y = 32;
    }
}

/// Returned when the current platform has no native popup implementation yet.
///
/// Native popups are separate from in-window popovers. Callers must report this
/// capability error or explicitly choose an in-window presentation.
#[derive(Debug, Error)]
#[error("popups are not supported on this platform")]
pub struct PopupNotSupportedError;

/// Resolve popup geometry in one logical coordinate space. The caller converts
/// the parent's anchor and the display work area into this space before calling.
pub fn popup_bounds(
    options: &PopupOptions,
    content: Size<Pixels>,
    work_area: Bounds<Pixels>,
) -> Bounds<Pixels> {
    fn anchor_fraction(anchor: PopupAnchor) -> (f32, f32) {
        use PopupAnchor::*;
        match anchor {
            TopLeft => (0.0, 0.0),
            Top => (0.5, 0.0),
            TopRight => (1.0, 0.0),
            Left => (0.0, 0.5),
            Center => (0.5, 0.5),
            Right => (1.0, 0.5),
            BottomLeft => (0.0, 1.0),
            Bottom => (0.5, 1.0),
            BottomRight => (1.0, 1.0),
        }
    }
    fn gravity_fraction(gravity: PopupGravity) -> (f32, f32) {
        use PopupGravity::*;
        match gravity {
            TopLeft => (1.0, 1.0),
            Top => (0.5, 1.0),
            TopRight => (0.0, 1.0),
            Left => (1.0, 0.5),
            Center => (0.5, 0.5),
            Right => (0.0, 0.5),
            BottomLeft => (1.0, 0.0),
            Bottom => (0.5, 0.0),
            BottomRight => (0.0, 0.0),
        }
    }
    fn axis(
        anchor_rect: std::ops::Range<f32>,
        anchor: f32,
        gravity: f32,
        offset: f32,
        length: f32,
        work_area: std::ops::Range<f32>,
        (flip, slide, resize): (bool, bool, bool),
    ) -> (f32, f32) {
        let extent = anchor_rect.end - anchor_rect.start;
        let available = work_area.end - work_area.start;
        let length = if resize {
            length.min(available)
        } else {
            length
        };
        let start = anchor_rect.start + extent * anchor - length * gravity + offset;
        let overflow =
            |p: f32| (work_area.start - p).max(0.0) + (p + length - work_area.end).max(0.0);
        let opposite =
            anchor_rect.start + extent * (1.0 - anchor) - length * (1.0 - gravity) - offset;
        let start = if flip && overflow(opposite) < overflow(start) {
            opposite
        } else {
            start
        };
        (
            if slide {
                start.clamp(
                    work_area.start,
                    (work_area.end - length).max(work_area.start),
                )
            } else {
                start
            },
            length,
        )
    }
    let (ax, ay) = anchor_fraction(options.anchor);
    let (gx, gy) = gravity_fraction(options.gravity);
    let flags = options.constraint_adjustment;
    let (x, width) = axis(
        options.anchor_rect.left().as_f32()..options.anchor_rect.right().as_f32(),
        ax,
        gx,
        options.offset.x.into(),
        content.width.into(),
        work_area.left().as_f32()..work_area.right().as_f32(),
        (
            flags.contains(PopupConstraintAdjustment::FLIP_X),
            flags.contains(PopupConstraintAdjustment::SLIDE_X),
            flags.contains(PopupConstraintAdjustment::RESIZE_X),
        ),
    );
    let (y, height) = axis(
        options.anchor_rect.top().as_f32()..options.anchor_rect.bottom().as_f32(),
        ay,
        gy,
        options.offset.y.into(),
        content.height.into(),
        work_area.top().as_f32()..work_area.bottom().as_f32(),
        (
            flags.contains(PopupConstraintAdjustment::FLIP_Y),
            flags.contains(PopupConstraintAdjustment::SLIDE_Y),
            flags.contains(PopupConstraintAdjustment::RESIZE_Y),
        ),
    );
    Bounds::new(point(px(x), px(y)), size(px(width), px(height)))
}

/// Choose the display with greatest anchor overlap, then nearest center when
/// the anchor is in a gap. Coordinates must use one platform coordinate space.
pub fn popup_display(anchor: Bounds<Pixels>, displays: &[Bounds<Pixels>]) -> Option<usize> {
    let center = anchor.center();
    let score = |display: &Bounds<Pixels>| {
        let overlap = anchor.intersect(display);
        let area = overlap.size.width.as_f32().max(0.0) * overlap.size.height.as_f32().max(0.0);
        let x = center
            .x
            .as_f32()
            .clamp(display.left().as_f32(), display.right().as_f32());
        let y = center
            .y
            .as_f32()
            .clamp(display.top().as_f32(), display.bottom().as_f32());
        let distance = (center.x.as_f32() - x).powi(2) + (center.y.as_f32() - y).powi(2);
        (area, distance)
    };
    displays
        .iter()
        .enumerate()
        .filter(|(_, display)| display.size.width > px(0.0) && display.size.height > px(0.0))
        .max_by(|(_, a), (_, b)| {
            let a = score(a);
            let b = score(b);
            a.0.total_cmp(&b.0).then_with(|| b.1.total_cmp(&a.1))
        })
        .map(|(index, _)| index)
}
