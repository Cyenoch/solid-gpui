use gpui::{BoxShadow as GpuiBoxShadow, SharedString, Styled, point, px, rgba};

use crate::protocol::{
    AlignItemsCode, AlignSelfCode, CursorCode, FlexDirectionCode, FontStyleCode, FontWeightCode,
    JustifyContentCode, OverflowCode, PositionCode, Style, TextAlignCode, TextDecorationCode,
    TextOverflowCode,
};

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
    if style.grid_columns.is_some() || style.grid_rows.is_some() {
        element = element.grid();
        if let Some(columns) = style.grid_columns {
            element = element.grid_cols(columns as u16);
        }
        if let Some(rows) = style.grid_rows {
            element = element.grid_rows(rows as u16);
        }
    } else if style.flex_direction.is_some()
        || style.gap.is_some()
        || style.justify_content.is_some()
        || style.align_items.is_some()
    {
        element = element.flex();
        if style.flex_direction.is_none() {
            element = element.flex_col();
        }
    }
    if let Some(span) = style.grid_column_span {
        element = element.col_span(span as u16);
    }
    if let Some(span) = style.grid_row_span {
        element = element.row_span(span as u16);
    }
    if let Some(width) = style.width {
        element = element.w(px(width));
    }
    if let Some(height) = style.height {
        element = element.h(px(height));
    }
    if let Some(position) = style.position {
        element = if position == PositionCode::Absolute {
            element.absolute()
        } else {
            element.relative()
        };
    }
    if style.position != Some(PositionCode::Overlay) {
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
            FlexDirectionCode::Row => element.flex_row(),
            FlexDirectionCode::Column => element.flex_col(),
            FlexDirectionCode::RowReverse => element.flex_row_reverse(),
            FlexDirectionCode::ColumnReverse => element.flex_col_reverse(),
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
            JustifyContentCode::FlexStart => element.justify_start(),
            JustifyContentCode::Center => element.justify_center(),
            JustifyContentCode::FlexEnd => element.justify_end(),
            JustifyContentCode::SpaceBetween => element.justify_between(),
            JustifyContentCode::SpaceAround => element.justify_around(),
            JustifyContentCode::SpaceEvenly => element.justify_evenly(),
        };
    }
    if let Some(align) = style.align_items {
        element = match align {
            AlignItemsCode::FlexStart => element.items_start(),
            AlignItemsCode::Center => element.items_center(),
            AlignItemsCode::FlexEnd => element.items_end(),
            AlignItemsCode::Stretch => element.items_stretch(),
            AlignItemsCode::Baseline => element.items_baseline(),
        };
    }
    if let Some(align_self) = style.align_self {
        element = match align_self {
            AlignSelfCode::Start => element.self_start(),
            AlignSelfCode::End => element.self_end(),
            AlignSelfCode::FlexStart => element.self_flex_start(),
            AlignSelfCode::FlexEnd => element.self_flex_end(),
            AlignSelfCode::Center => element.self_center(),
            AlignSelfCode::Baseline => element.self_baseline(),
            AlignSelfCode::Stretch => element.self_stretch(),
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
    if let Some(value) = style.padding_top {
        element = element.pt(px(value));
    }
    if let Some(value) = style.padding_right {
        element = element.pr(px(value));
    }
    if let Some(value) = style.padding_bottom {
        element = element.pb(px(value));
    }
    if let Some(value) = style.padding_left {
        element = element.pl(px(value));
    }
    if let Some(value) = style.border_top_width {
        element = element.border_t(px(value));
    }
    if let Some(value) = style.border_right_width {
        element = element.border_r(px(value));
    }
    if let Some(value) = style.border_bottom_width {
        element = element.border_b(px(value));
    }
    if let Some(value) = style.border_left_width {
        element = element.border_l(px(value));
    }
    if let Some(value) = style.border_top_left_radius {
        element = element.rounded_tl(px(value));
    }
    if let Some(value) = style.border_top_right_radius {
        element = element.rounded_tr(px(value));
    }
    if let Some(value) = style.border_bottom_right_radius {
        element = element.rounded_br(px(value));
    }
    if let Some(value) = style.border_bottom_left_radius {
        element = element.rounded_bl(px(value));
    }
    if let Some(value) = style.width_percent {
        element = element.w(gpui::relative(value / 100.));
    }
    if let Some(value) = style.height_percent {
        element = element.h(gpui::relative(value / 100.));
    }
    if let Some(value) = style.flex_wrap {
        element = element.flex();
        element.style().flex_wrap = Some(match value {
            crate::protocol::FlexWrapCode::NoWrap => gpui::FlexWrap::NoWrap,
            crate::protocol::FlexWrapCode::Wrap => gpui::FlexWrap::Wrap,
            crate::protocol::FlexWrapCode::WrapReverse => gpui::FlexWrap::WrapReverse,
        });
    }
    if super::border::has_edge_colors(style) {
        element = element.border_color(gpui::transparent_black());
    }
    if let Some(overflow) = style.overflow {
        element = match overflow {
            OverflowCode::Visible => {
                element.style().overflow.x = Some(gpui::Overflow::Visible);
                element.style().overflow.y = Some(gpui::Overflow::Visible);
                element
            }
            OverflowCode::Hidden => {
                element.style().overflow.x = Some(gpui::Overflow::Hidden);
                element.style().overflow.y = Some(gpui::Overflow::Hidden);
                element
            }
            OverflowCode::Scroll => {
                element.style().overflow.x = Some(gpui::Overflow::Scroll);
                element.style().overflow.y = Some(gpui::Overflow::Scroll);
                element
            }
        };
    }
    if let Some(background) = style.background_rgba {
        element = element.bg(rgba(background));
    }
    if let Some(gradient) = style.linear_gradient {
        element = element.bg(gpui::linear_gradient(
            gradient.angle,
            gpui::linear_color_stop(rgba(gradient.start_color), gradient.start_position),
            gpui::linear_color_stop(rgba(gradient.end_color), gradient.end_position),
        ));
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
    if with_cursor && let Some(cursor) = style.cursor {
        element = element.cursor(match cursor {
            CursorCode::Default => gpui::CursorStyle::Arrow,
            CursorCode::Text => gpui::CursorStyle::IBeam,
            CursorCode::Pointer => gpui::CursorStyle::PointingHand,
            CursorCode::Grab => gpui::CursorStyle::OpenHand,
            CursorCode::Grabbing | CursorCode::Move => gpui::CursorStyle::ClosedHand,
            CursorCode::NotAllowed | CursorCode::NoDrop => gpui::CursorStyle::OperationNotAllowed,
            CursorCode::ContextMenu => gpui::CursorStyle::ContextualMenu,
            CursorCode::Crosshair => gpui::CursorStyle::Crosshair,
            CursorCode::VerticalText => gpui::CursorStyle::IBeamCursorForVerticalLayout,
            CursorCode::Alias => gpui::CursorStyle::DragLink,
            CursorCode::Copy => gpui::CursorStyle::DragCopy,
            CursorCode::EwResize => gpui::CursorStyle::ResizeLeftRight,
            CursorCode::NsResize => gpui::CursorStyle::ResizeUpDown,
            CursorCode::NeswResize => gpui::CursorStyle::ResizeUpLeftDownRight,
            CursorCode::NwseResize => gpui::CursorStyle::ResizeUpRightDownLeft,
            CursorCode::ColResize => gpui::CursorStyle::ResizeColumn,
            CursorCode::RowResize => gpui::CursorStyle::ResizeRow,
        });
    }
    apply_text_style(element, Some(style))
}

pub(super) fn apply_text_style<E: Styled>(mut element: E, style: Option<&Style>) -> E {
    let Some(style) = style else { return element };
    if let Some(font_family) = style.font_family.as_ref() {
        element = element.font_family(SharedString::from(font_family.clone()));
    }
    if let Some(font_style) = style.font_style {
        element = match font_style {
            FontStyleCode::Normal => element.not_italic(),
            FontStyleCode::Italic => element.italic(),
        };
    }
    if let Some(text_decoration) = style.text_decoration {
        let text_style = element.text_style();
        text_style.underline = None;
        text_style.strikethrough = None;
        element = match text_decoration {
            TextDecorationCode::None => element.text_decoration_none(),
            TextDecorationCode::Underline => element.underline(),
            TextDecorationCode::LineThrough => element.line_through(),
        };
    }
    if let Some(line_height) = style.line_height {
        element = element.line_height(px(line_height));
    }
    if let Some(align) = style.text_align {
        element = match align {
            TextAlignCode::Left => element.text_left(),
            TextAlignCode::Center => element.text_center(),
            TextAlignCode::Right => element.text_right(),
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
            TextOverflowCode::Clip => {
                element.text_style().text_overflow = None;
                element
            }
            TextOverflowCode::Ellipsis => element.text_ellipsis(),
        };
    }
    if let Some(size) = style.font_size {
        element = element.text_size(px(size));
    }
    if let Some(weight) = style.font_weight {
        element = element.font_weight(match weight {
            FontWeightCode::Normal => gpui::FontWeight::NORMAL,
            FontWeightCode::Medium => gpui::FontWeight::MEDIUM,
            FontWeightCode::Semibold => gpui::FontWeight::SEMIBOLD,
            FontWeightCode::Bold => gpui::FontWeight::BOLD,
            FontWeightCode::Heavy => gpui::FontWeight::BLACK,
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
            text_style.font_style = match font_style {
                FontStyleCode::Normal => gpui::FontStyle::Normal,
                FontStyleCode::Italic => gpui::FontStyle::Italic,
            };
        }
        if let Some(text_decoration) = style.text_decoration {
            text_style.underline = match text_decoration {
                TextDecorationCode::Underline => Some(gpui::UnderlineStyle::default()),
                TextDecorationCode::None | TextDecorationCode::LineThrough => None,
            };
            text_style.strikethrough = match text_decoration {
                TextDecorationCode::LineThrough => Some(gpui::StrikethroughStyle::default()),
                TextDecorationCode::None | TextDecorationCode::Underline => None,
            };
        }
        if let Some(weight) = style.font_weight {
            text_style.font_weight = match weight {
                FontWeightCode::Normal => gpui::FontWeight::NORMAL,
                FontWeightCode::Medium => gpui::FontWeight::MEDIUM,
                FontWeightCode::Semibold => gpui::FontWeight::SEMIBOLD,
                FontWeightCode::Bold => gpui::FontWeight::BOLD,
                FontWeightCode::Heavy => gpui::FontWeight::BLACK,
            };
        }
    }
    text_style.to_run(len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Style;
    use gpui::{CursorStyle, Styled};

    #[test]
    fn cursor_style_respects_without_cursor() {
        let style = Style {
            cursor: Some(CursorCode::Pointer),
            ..Style::default()
        };

        let mut styled = apply_style(gpui::div(), Some(&style));
        assert_eq!(styled.style().mouse_cursor, Some(CursorStyle::PointingHand));

        let mut styled = apply_style_without_cursor(gpui::div(), Some(&style));
        assert_eq!(styled.style().mouse_cursor, None);
    }
    #[test]
    fn migration_layout_overrides_shorthands_without_extra_boxes() {
        let style = Style {
            padding: Some(12.),
            padding_left: Some(88.),
            padding_top: Some(0.),
            border_radius: Some(8.),
            border_top_right_radius: Some(0.),
            border_width: Some(0.),
            border_bottom_width: Some(1.),
            width_percent: Some(50.),
            flex_wrap: Some(crate::protocol::FlexWrapCode::Wrap),
            ..Style::default()
        };
        let mut element = apply_style(gpui::div(), Some(&style));
        let actual = element.style();
        assert_eq!(actual.padding.left, Some(px(88.).into()));
        assert_eq!(actual.padding.top, Some(px(0.).into()));
        assert_eq!(actual.border_widths.bottom, Some(px(1.).into()));
        assert_eq!(actual.corner_radii.top_right, Some(px(0.).into()));
        assert_eq!(actual.size.width, Some(gpui::relative(0.5).into()));
        assert_eq!(actual.flex_wrap, Some(gpui::FlexWrap::Wrap));
    }
}
