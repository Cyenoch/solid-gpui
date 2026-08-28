use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct NodeWire(
    u32,
    u32,
    u32,
    u32,
    Option<StyleWire>,
    Option<String>,
    u32,
    Option<HostPropertiesWire>,
    Option<AccessibilityWire>,
    bool,
    #[serde(default, skip_serializing_if = "Option::is_none")] Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")] Option<String>,
);
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum HostPropertiesWire {
    TextInput(TextInputWire),
    VirtualList(VirtualListWire),
    Image(ImageWire),
    Drag(DragWire),
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct TextInputWire(
    u32,
    String,
    Option<String>,
    bool,
    bool,
    bool,
    u32,
    u32,
    u32,
    Option<u32>,
    Option<u32>,
    Option<u32>,
    bool,
);

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct VirtualListWire(u32, u32, u32, u32, f32, u32);
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ImageWire(u32, String, u32, Option<String>);
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct DragWire(u32, Option<String>, Option<Vec<String>>, bool, bool);
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct AccessibilityWire(
    u32,
    Option<String>,
    Option<String>,
    bool,
    Option<bool>,
    Option<bool>,
    Option<String>,
);
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct BoxShadowValueWire(f32, f32, f32, f32, u32, u32);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum BoxShadowWire {
    Single(u32, BoxShadowValueWire),
    Double(u32, [BoxShadowValueWire; 2]),
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct StyleWire(
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<TransitionWire>,
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<f32>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<u32>,
    Option<BoxShadowWire>,
    Option<String>,
);

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct TransitionWire(u32, u32, u32, u32);
impl From<&Node> for NodeWire {
    fn from(node: &Node) -> Self {
        Self(
            node.id,
            node.parent_id,
            node.index,
            node.kind,
            node.style.as_ref().map(StyleWire::from),
            node.text.clone(),
            node.listener_id,
            node.host_properties.as_ref().map(HostPropertiesWire::from),
            node.accessibility.as_ref().map(AccessibilityWire::from),
            node.focusable,
            (node.selectable || node.tooltip.is_some()).then_some(node.selectable),
            node.tooltip.clone(),
        )
    }
}
impl TryFrom<NodeWire> for Node {
    type Error = ProtocolError;

    fn try_from(node: NodeWire) -> Result<Self, Self::Error> {
        if let Some(style) = node.4.as_ref() {
            validate_style_wire(style)?;
        }
        let selectable = node.10.unwrap_or(false);
        if selectable && node.3 != crate::tree::KIND_TEXT {
            return Err(ProtocolError::InvalidHostProperties);
        }
        let host_properties = node.7.map(HostProperties::try_from).transpose()?;
        validate_host_kind(node.3, host_properties.as_ref())?;
        let tooltip = node.11;
        if tooltip
            .as_ref()
            .is_some_and(|value| !valid_tooltip_text(value))
        {
            return Err(ProtocolError::InvalidHostProperties);
        }
        Ok(Self {
            id: node.0,
            parent_id: node.1,
            index: node.2,
            kind: node.3,
            style: node.4.map(Style::from),
            text: node.5,
            listener_id: node.6,
            host_properties,
            accessibility: node.8.map(AccessibilityProperties::from),
            focusable: node.9,
            selectable,
            tooltip,
        })
    }
}

impl From<&BoxShadow> for BoxShadowValueWire {
    fn from(value: &BoxShadow) -> Self {
        Self(
            value.offset_x,
            value.offset_y,
            value.blur_radius,
            value.spread_radius,
            value.color_rgba,
            u32::from(value.inset),
        )
    }
}

fn box_shadow_wire(shadows: Option<&[BoxShadow]>) -> Option<BoxShadowWire> {
    match shadows {
        Some([shadow]) => Some(BoxShadowWire::Single(1, BoxShadowValueWire::from(shadow))),
        Some([first, second]) => Some(BoxShadowWire::Double(
            2,
            [
                BoxShadowValueWire::from(first),
                BoxShadowValueWire::from(second),
            ],
        )),
        _ => None,
    }
}

impl From<BoxShadowWire> for Vec<BoxShadow> {
    fn from(value: BoxShadowWire) -> Self {
        let values = match value {
            BoxShadowWire::Single(_, value) => vec![value],
            BoxShadowWire::Double(_, values) => values.into_iter().collect(),
        };
        values
            .into_iter()
            .map(|value| BoxShadow {
                offset_x: value.0,
                offset_y: value.1,
                blur_radius: value.2,
                spread_radius: value.3,
                color_rgba: value.4,
                inset: value.5 != 0,
            })
            .collect()
    }
}

impl From<&Style> for StyleWire {
    fn from(style: &Style) -> Self {
        Self(
            style.width,
            style.height,
            style.flex_direction,
            style.flex_grow,
            style.padding,
            style.gap,
            style.background_rgba,
            style.color_rgba,
            style.opacity,
            style.transition.as_ref().map(TransitionWire::from),
            style.justify_content,
            style.align_items,
            style.border_radius,
            style.border_width,
            style.border_color_rgba,
            style.font_size,
            style.font_weight,
            style.overflow,
            style.line_clamp,
            style.text_overflow,
            style.margin_top,
            style.margin_right,
            style.margin_bottom,
            style.margin_left,
            style.font_style,
            style.text_decoration,
            style.line_height,
            style.min_width,
            style.max_width,
            style.min_height,
            style.max_height,
            style.flex_shrink,
            style.align_self,
            style.position,
            style.left,
            style.top,
            style.right,
            style.bottom,
            style.cursor,
            style.text_align,
            box_shadow_wire(style.box_shadows.as_deref()),
            style.font_family.clone(),
        )
    }
}

impl From<StyleWire> for Style {
    fn from(style: StyleWire) -> Self {
        Self {
            width: style.0,
            height: style.1,
            flex_direction: style.2,
            flex_grow: style.3,
            padding: style.4,
            gap: style.5,
            background_rgba: style.6,
            color_rgba: style.7,
            opacity: style.8,
            transition: style.9.map(Transition::from),
            justify_content: style.10,
            align_items: style.11,
            border_radius: style.12,
            border_width: style.13,
            border_color_rgba: style.14,
            font_size: style.15,
            font_weight: style.16,
            overflow: style.17,
            line_clamp: style.18,
            text_overflow: style.19,
            margin_top: style.20,
            margin_right: style.21,
            margin_bottom: style.22,
            margin_left: style.23,
            font_style: style.24,
            text_decoration: style.25,
            line_height: style.26,
            min_width: style.27,
            max_width: style.28,
            min_height: style.29,
            max_height: style.30,
            flex_shrink: style.31,
            align_self: style.32,
            position: style.33,
            left: style.34,
            top: style.35,
            right: style.36,
            bottom: style.37,
            cursor: style.38,
            text_align: style.39,
            box_shadows: style.40.map(Vec::<BoxShadow>::from),
            font_family: style.41,
        }
    }
}

impl From<&Transition> for TransitionWire {
    fn from(transition: &Transition) -> Self {
        Self(
            transition.duration_ms,
            transition.delay_ms,
            transition.easing as u32,
            transition.properties,
        )
    }
}

impl From<TransitionWire> for Transition {
    fn from(transition: TransitionWire) -> Self {
        Self {
            duration_ms: transition.0,
            delay_ms: transition.1,
            easing: match transition.2 {
                0 => Easing::Linear,
                1 => Easing::EaseIn,
                2 => Easing::EaseOut,
                _ => Easing::EaseInOut,
            },
            properties: transition.3,
        }
    }
}

impl From<&HostProperties> for HostPropertiesWire {
    fn from(value: &HostProperties) -> Self {
        match value {
            HostProperties::TextInput(value) => Self::TextInput(TextInputWire::from(value)),
            HostProperties::VirtualList(value) => Self::VirtualList(VirtualListWire::from(value)),
            HostProperties::Image(value) => Self::Image(ImageWire::from(value)),
            HostProperties::Drag(value) => Self::Drag(DragWire(
                4,
                value.drag_type.clone(),
                value.export_files.clone(),
                value.accepts_drag_over,
                value.accepts_drop,
            )),
        }
    }
}

fn valid_image_source(source: &str) -> bool {
    !source.is_empty() && source.len() <= 1024 && !source.chars().any(char::is_control)
}

pub(super) fn valid_drag_type(drag_type: Option<&str>) -> bool {
    drag_type.is_none_or(|value| {
        !value.is_empty() && value.chars().count() <= 128 && !value.chars().any(char::is_control)
    })
}

fn valid_export_files(files: Option<&[String]>) -> bool {
    files.is_none_or(|files| {
        !files.is_empty() && files.len() <= 8 && files.iter().all(|path| valid_image_source(path))
    })
}

fn validate_text_input(value: TextInputProperties) -> Result<HostProperties, ProtocolError> {
    if value.selection_start > value.selection_end
        || value.marked_start.is_some() != value.marked_end.is_some()
        || value
            .marked_start
            .zip(value.marked_end)
            .is_some_and(|(start, end)| start > end)
    {
        return Err(ProtocolError::InvalidHostProperties);
    }
    Ok(HostProperties::TextInput(value))
}

impl TryFrom<HostPropertiesWire> for HostProperties {
    type Error = ProtocolError;

    fn try_from(value: HostPropertiesWire) -> Result<Self, Self::Error> {
        match value {
            HostPropertiesWire::TextInput(value) => {
                validate_text_input(TextInputProperties::from(value))
            }
            HostPropertiesWire::VirtualList(value) if value.0 == 2 => {
                if value.2 > value.3 || value.3 > value.1 || !value.4.is_finite() || value.4 <= 0.0
                {
                    return Err(ProtocolError::InvalidHostProperties);
                }
                Ok(Self::VirtualList(VirtualListProperties::from(value)))
            }
            HostPropertiesWire::Image(value)
                if value.0 == 3
                    && valid_image_source(&value.1)
                    && value.3.as_deref().is_none_or(valid_image_source)
                    && (1..=5).contains(&value.2) =>
            {
                Ok(Self::Image(ImageProperties {
                    source: value.1,
                    object_fit: value.2,
                    fallback_source: value.3,
                }))
            }
            HostPropertiesWire::Drag(value)
                if value.0 == 4
                    && valid_drag_type(value.1.as_deref())
                    && valid_export_files(value.2.as_deref()) =>
            {
                Ok(Self::Drag(DragProperties {
                    drag_type: value.1,
                    export_files: value.2,
                    accepts_drag_over: value.3,
                    accepts_drop: value.4,
                }))
            }
            _ => Err(ProtocolError::InvalidHostProperties),
        }
    }
}
pub(super) fn valid_tooltip_text(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}

fn validate_host_kind(
    kind: u32,
    host_properties: Option<&HostProperties>,
) -> Result<(), ProtocolError> {
    match (kind, host_properties) {
        (1 | 3, Some(HostProperties::Drag(_)))
        | (5, Some(HostProperties::TextInput(_)))
        | (6, Some(HostProperties::VirtualList(_)))
        | (7, Some(HostProperties::Image(_))) => Ok(()),
        (5..=7, None) => Err(ProtocolError::InvalidHostProperties),
        (_, Some(_)) => Err(ProtocolError::InvalidHostProperties),
        (_, None) => Ok(()),
    }
}

fn valid_font_family(value: &str) -> bool {
    !value.is_empty() && value.chars().count() <= 64 && !value.chars().any(char::is_control)
}

fn valid_box_shadow_value(value: &BoxShadowValueWire) -> bool {
    value.0.is_finite()
        && value.1.is_finite()
        && value.2.is_finite()
        && value.2 >= 0.0
        && value.3.is_finite()
        && value.3 >= 0.0
        && value.5 <= 1
}

fn valid_box_shadow(value: &BoxShadowWire) -> bool {
    match value {
        BoxShadowWire::Single(tag, value) => *tag == 1 && valid_box_shadow_value(value),
        BoxShadowWire::Double(tag, values) => {
            *tag == 2 && values.iter().all(valid_box_shadow_value)
        }
    }
}

pub(super) fn validate_style_wire(style: &StyleWire) -> Result<(), ProtocolError> {
    if style.2.is_some_and(|direction| direction > 4)
        || style.33.is_some_and(|position| position > 2)
        || (style.33 == Some(2) && (style.36.is_some() || style.37.is_some()))
        || style.38.is_some_and(|cursor| cursor > 18)
        || style.39.is_some_and(|align| align > 3)
        || [style.34, style.35, style.36, style.37]
            .into_iter()
            .flatten()
            .any(|value| !value.is_finite())
        || style.10.is_some_and(|justify| !(1..=6).contains(&justify))
        || style.11.is_some_and(|align| !(1..=5).contains(&align))
        || style
            .17
            .is_some_and(|overflow| !(1..=3).contains(&overflow))
        || style
            .18
            .is_some_and(|line_clamp| !(1..=100).contains(&line_clamp))
        || style
            .19
            .is_some_and(|text_overflow| !matches!(text_overflow, 1 | 2))
        || style
            .24
            .is_some_and(|font_style| !matches!(font_style, 0 | 1))
        || style
            .25
            .is_some_and(|text_decoration| !matches!(text_decoration, 0..=2))
        || style
            .32
            .is_some_and(|align_self| !(1..=7).contains(&align_self))
        || [
            style.0, style.1, style.3, style.4, style.5, style.8, style.12, style.13, style.20,
            style.21, style.22, style.23, style.26, style.27, style.28, style.29, style.30,
            style.31,
        ]
        .into_iter()
        .flatten()
        .any(|value| !value.is_finite() || value < 0.0)
        || style.8.is_some_and(|opacity| opacity > 1.0)
        || style
            .15
            .is_some_and(|size| !size.is_finite() || size <= 0.0)
        || style
            .16
            .is_some_and(|weight| !matches!(weight, 400 | 500 | 600 | 700 | 900))
        || style
            .40
            .as_ref()
            .is_some_and(|shadow| !valid_box_shadow(shadow))
        || style
            .41
            .as_ref()
            .is_some_and(|family| !valid_font_family(family))
    {
        return Err(ProtocolError::InvalidStyle);
    }
    if let Some(transition) = style.9.as_ref()
        && (transition.2 > 3
            || transition.3 == 0
            || transition.3
                & !(TRANSITION_OPACITY
                    | TRANSITION_BACKGROUND_COLOR
                    | TRANSITION_WIDTH
                    | TRANSITION_HEIGHT)
                != 0)
    {
        return Err(ProtocolError::InvalidStyle);
    }
    Ok(())
}

impl From<&TextInputProperties> for TextInputWire {
    fn from(value: &TextInputProperties) -> Self {
        Self(
            1,
            value.value.clone(),
            value.placeholder.clone(),
            value.multiline,
            value.disabled,
            value.controlled,
            value.ack_edit_seq,
            value.selection_start,
            value.selection_end,
            value.marked_start,
            value.marked_end,
            value.max_length,
            value.selection_reversed,
        )
    }
}
impl From<&ImageProperties> for ImageWire {
    fn from(value: &ImageProperties) -> Self {
        Self(
            3,
            value.source.clone(),
            value.object_fit,
            value.fallback_source.clone(),
        )
    }
}

impl From<TextInputWire> for TextInputProperties {
    fn from(value: TextInputWire) -> Self {
        Self {
            value: value.1,
            placeholder: value.2,
            multiline: value.3,
            disabled: value.4,
            controlled: value.5,
            ack_edit_seq: value.6,
            selection_start: value.7,
            selection_end: value.8,
            marked_start: value.9,
            marked_end: value.10,
            max_length: value.11,
            selection_reversed: value.12,
        }
    }
}

impl From<&VirtualListProperties> for VirtualListWire {
    fn from(value: &VirtualListProperties) -> Self {
        Self(
            2,
            value.item_count,
            value.range_start,
            value.range_end,
            value.estimated_item_size,
            value.overscan,
        )
    }
}

impl From<VirtualListWire> for VirtualListProperties {
    fn from(value: VirtualListWire) -> Self {
        Self {
            item_count: value.1,
            range_start: value.2,
            range_end: value.3,
            estimated_item_size: value.4,
            overscan: value.5,
        }
    }
}

impl From<&AccessibilityProperties> for AccessibilityWire {
    fn from(value: &AccessibilityProperties) -> Self {
        Self(
            value.role,
            value.label.clone(),
            value.description.clone(),
            value.disabled,
            value.checked,
            value.selected,
            value.value.clone(),
        )
    }
}

impl From<AccessibilityWire> for AccessibilityProperties {
    fn from(value: AccessibilityWire) -> Self {
        Self {
            role: value.0,
            label: value.1,
            description: value.2,
            disabled: value.3,
            checked: value.4,
            selected: value.5,
            value: value.6,
        }
    }
}
