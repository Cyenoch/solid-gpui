use gpui::{BoxShadow as GpuiBoxShadow, SharedString, Styled, point, px, rgba};

use crate::protocol::Style;

pub(super) fn apply_style<E: Styled>(element: E, style: Option<&Style>) -> E {
    apply_style_with_cursor(element, style, true)
}

pub(super) fn apply_style_without_cursor<E: Styled>(element: E, style: Option<&Style>) -> E {
    apply_style_with_cursor(element, style, false)
}

fn apply_style_with_cursor<E: Styled>(
    mut element: E,
    style: Option<&Style>,
    with_cursor: bool,
) -> E {
    let Some(style) = style else { return element };
    if style.flex_direction.is_some()
        || style.gap.is_some()
        || style.justify_content.is_some()
        || style.align_items.is_some()
    {
        element = element.flex();
        if style.flex_direction.is_none() {
            element = element.flex_col();
        }
    }
    if let Some(width) = style.width {
        element = element.w(px(width));
    }
    if let Some(height) = style.height {
        element = element.h(px(height));
    }
    if let Some(position) = style.position {
        element = if position == 1 {
            element.absolute()
        } else {
            element.relative()
        };
    }
    if style.position != Some(2) {
        if let Some(left) = style.left {
            element = element.left(px(left));
        }
        if let Some(top) = style.top {
            element = element.top(px(top));
        }
        if let Some(right) = style.right {
            element = element.right(px(right));
        }
        if let Some(bottom) = style.bottom {
            element = element.bottom(px(bottom));
        }
    }
    if let Some(direction) = style.flex_direction {
        element = match direction {
            1 => element.flex_row(),
            2 => element.flex_col(),
            3 => element.flex_row_reverse(),
            4 => element.flex_col_reverse(),
            _ => element,
        };
    }
    if let Some(grow) = style.flex_grow {
        element.style().flex_grow = Some(grow);
    }
    if let Some(padding) = style.padding {
        element = element.p(px(padding));
    }
    if let Some(gap) = style.gap {
        element = element.gap(px(gap));
    }
    if let Some(margin) = style.margin_top {
        element = element.mt(px(margin));
    }
    if let Some(margin) = style.margin_right {
        element = element.mr(px(margin));
    }
    if let Some(margin) = style.margin_bottom {
        element = element.mb(px(margin));
    }
    if let Some(margin) = style.margin_left {
        element = element.ml(px(margin));
    }
    if let Some(min_width) = style.min_width {
        element = element.min_w(px(min_width));
    }
    if let Some(max_width) = style.max_width {
        element = element.max_w(px(max_width));
    }
    if let Some(min_height) = style.min_height {
        element = element.min_h(px(min_height));
    }
    if let Some(max_height) = style.max_height {
        element = element.max_h(px(max_height));
    }
    if let Some(shrink) = style.flex_shrink {
        element.style().flex_shrink = Some(shrink);
    }
    if let Some(justify) = style.justify_content {
        element = match justify {
            1 => element.justify_start(),
            2 => element.justify_center(),
            3 => element.justify_end(),
            4 => element.justify_between(),
            5 => element.justify_around(),
            6 => element.justify_evenly(),
            _ => element,
        };
    }
    if let Some(align) = style.align_items {
        element = match align {
            1 => element.items_start(),
            2 => element.items_center(),
            3 => element.items_end(),
            4 => element.items_stretch(),
            5 => element.items_baseline(),
            _ => element,
        };
    }
    if let Some(align_self) = style.align_self {
        element = match align_self {
            1 => element.self_start(),
            2 => element.self_end(),
            3 => element.self_flex_start(),
            4 => element.self_flex_end(),
            5 => element.self_center(),
            6 => element.self_baseline(),
            7 => element.self_stretch(),
            _ => element,
        };
    }
    if let Some(radius) = style.border_radius {
        element = element.rounded(px(radius));
    }
    if let Some(width) = style.border_width {
        element = element.border(px(width));
    }
    if let Some(color) = style.border_color_rgba {
        element = element.border_color(rgba(color));
    }
    if let Some(overflow) = style.overflow {
        element = match overflow {
            1 => {
                element.style().overflow.x = Some(gpui::Overflow::Visible);
                element.style().overflow.y = Some(gpui::Overflow::Visible);
                element
            }
            2 => {
                element.style().overflow.x = Some(gpui::Overflow::Hidden);
                element.style().overflow.y = Some(gpui::Overflow::Hidden);
                element
            }
            3 => {
                element.style().overflow.x = Some(gpui::Overflow::Scroll);
                element.style().overflow.y = Some(gpui::Overflow::Scroll);
                element
            }
            _ => element,
        };
    }
    if let Some(background) = style.background_rgba {
        element = element.bg(rgba(background));
    }
    if let Some(shadows) = style.box_shadows.as_ref() {
        element = element.shadow(
            shadows
                .iter()
                .map(|shadow| GpuiBoxShadow {
                    color: rgba(shadow.color_rgba).into(),
                    offset: point(px(shadow.offset_x), px(shadow.offset_y)),
                    blur_radius: px(shadow.blur_radius),
                    spread_radius: px(shadow.spread_radius),
                    inset: shadow.inset,
                })
                .collect(),
        );
    }
    if let Some(color) = style.color_rgba {
        element = element.text_color(rgba(color));
    }
    if let Some(opacity) = style.opacity {
        element = element.opacity(opacity);
    }
    if with_cursor {
        if let Some(cursor) = style.cursor {
            element = element.cursor(match cursor {
                0 => gpui::CursorStyle::Arrow,
                1 => gpui::CursorStyle::IBeam,
                2 => gpui::CursorStyle::PointingHand,
                3 => gpui::CursorStyle::OpenHand,
                4 | 12 => gpui::CursorStyle::ClosedHand,
                5 | 11 => gpui::CursorStyle::OperationNotAllowed,
                6 => gpui::CursorStyle::ContextualMenu,
                7 => gpui::CursorStyle::Crosshair,
                8 => gpui::CursorStyle::IBeamCursorForVerticalLayout,
                9 => gpui::CursorStyle::DragLink,
                10 => gpui::CursorStyle::DragCopy,
                13 => gpui::CursorStyle::ResizeLeftRight,
                14 => gpui::CursorStyle::ResizeUpDown,
                15 => gpui::CursorStyle::ResizeUpLeftDownRight,
                16 => gpui::CursorStyle::ResizeUpRightDownLeft,
                17 => gpui::CursorStyle::ResizeColumn,
                18 => gpui::CursorStyle::ResizeRow,
                _ => gpui::CursorStyle::Arrow,
            });
        }
    }
    element
}

pub(super) fn apply_text_style<E: Styled>(mut element: E, style: Option<&Style>) -> E {
    let Some(style) = style else { return element };
    if let Some(font_family) = style.font_family.as_ref() {
        element = element.font_family(SharedString::from(font_family.clone()));
    }
    if let Some(font_style) = style.font_style {
        element = match font_style {
            0 => element.not_italic(),
            1 => element.italic(),
            _ => element,
        };
    }
    if let Some(text_decoration) = style.text_decoration {
        let text_style = element.text_style();
        text_style.underline = None;
        text_style.strikethrough = None;
        element = match text_decoration {
            0 => element.text_decoration_none(),
            1 => element.underline(),
            2 => element.line_through(),
            _ => element,
        };
    }
    if let Some(line_height) = style.line_height {
        element = element.line_height(px(line_height));
    }
    if let Some(align) = style.text_align {
        element = match align {
            1 => element.text_left(),
            2 => element.text_center(),
            3 => element.text_right(),
            _ => element,
        };
    }
    if let Some(line_clamp) = style.line_clamp {
        element.text_style().line_clamp = Some(line_clamp as usize);
        if style.overflow.is_none() {
            element.style().overflow.x = Some(gpui::Overflow::Hidden);
            element.style().overflow.y = Some(gpui::Overflow::Hidden);
        }
    }
    if let Some(text_overflow) = style.text_overflow {
        element = match text_overflow {
            1 => {
                element.text_style().text_overflow = None;
                element
            }
            2 => element.text_ellipsis(),
            _ => element,
        };
    }
    if let Some(size) = style.font_size {
        element = element.text_size(px(size));
    }
    if let Some(weight) = style.font_weight {
        element = element.font_weight(match weight {
            400 => gpui::FontWeight::NORMAL,
            500 => gpui::FontWeight::MEDIUM,
            600 => gpui::FontWeight::SEMIBOLD,
            700 => gpui::FontWeight::BOLD,
            900 => gpui::FontWeight::BLACK,
            _ => gpui::FontWeight::NORMAL,
        });
    }
    element
}

pub(super) fn text_run(
    style: &gpui::TextStyle,
    node_style: Option<&Style>,
    len: usize,
) -> gpui::TextRun {
    let mut text_style = style.clone();
    if let Some(style) = node_style {
        if let Some(color) = style.color_rgba {
            text_style.color = rgba(color).into();
        }
        if let Some(font_family) = style.font_family.as_ref() {
            text_style.font_family = SharedString::from(font_family.clone());
        }
        if let Some(font_style) = style.font_style {
            text_style.font_style = if font_style == 1 {
                gpui::FontStyle::Italic
            } else {
                gpui::FontStyle::Normal
            };
        }
        if let Some(text_decoration) = style.text_decoration {
            text_style.underline =
                (text_decoration == 1).then_some(gpui::UnderlineStyle::default());
            text_style.strikethrough =
                (text_decoration == 2).then_some(gpui::StrikethroughStyle::default());
        }
        if let Some(weight) = style.font_weight {
            text_style.font_weight = match weight {
                400 => gpui::FontWeight::NORMAL,
                500 => gpui::FontWeight::MEDIUM,
                600 => gpui::FontWeight::SEMIBOLD,
                700 => gpui::FontWeight::BOLD,
                900 => gpui::FontWeight::BLACK,
                _ => gpui::FontWeight::NORMAL,
            };
        }
    }
    text_style.to_run(len)
}

#[cfg(test)]
mod tests {
    use super::{apply_style, apply_style_without_cursor};
    use crate::protocol::Style;
    use gpui::{CursorStyle, Styled};

    #[test]
    fn interactive_rich_text_omits_parent_cursor_only_when_requested() {
        let style = Style {
            cursor: Some(2),
            ..Style::default()
        };

        let mut styled = apply_style(gpui::div(), Some(&style));
        assert_eq!(styled.style().mouse_cursor, Some(CursorStyle::PointingHand));

        let mut styled = apply_style_without_cursor(gpui::div(), Some(&style));
        assert_eq!(styled.style().mouse_cursor, None);
    }
}
