use super::*;
use crate::protocol::{generated, generated_facts};
use bebop::{Record, SliceWrapper, SubRecord};
use std::sync::Arc;

fn structural_error() -> ProtocolError {
    ProtocolError::Decode(::bebop::DeserializeError::CorruptFrame)
}

fn wire_command_kind(value: u32) -> Result<generated::CommandKind, ProtocolError> {
    let kind = generated_facts::command_kind(value).ok_or(ProtocolError::UnknownCommand(value))?;
    if matches!(kind, generated::CommandKind::Unknown) {
        return Err(ProtocolError::UnknownCommand(value));
    }
    Ok(kind)
}

fn command_kind_value(value: generated::CommandKind) -> u32 {
    u8::from(value) as u32
}

fn wire_event_kind(value: u32) -> Result<generated::EventKind, ProtocolError> {
    let kind = generated_facts::event_kind(value).ok_or(ProtocolError::UnknownEvent(value))?;
    if matches!(kind, generated::EventKind::Unknown) {
        return Err(ProtocolError::UnknownEvent(value));
    }
    Ok(kind)
}

fn event_kind_value(value: generated::EventKind) -> u32 {
    u8::from(value) as u32
}

fn encode_envelope<'a>(body: generated::Body<'a>) -> Result<Vec<u8>, ProtocolError> {
    let envelope = generated::Envelope {
        protocol_version: Some(PROTOCOL_VERSION),
        body: Some(body),
    };
    let mut payload = Vec::with_capacity(envelope.serialized_size());
    envelope
        .serialize(&mut payload)
        .map_err(ProtocolError::Encode)?;
    if payload.len() > MAX_FRAME_LENGTH {
        return Err(ProtocolError::FrameTooLarge(payload.len()));
    }
    Ok(payload)
}
fn decode_envelope(payload: &[u8]) -> Result<generated::Envelope<'_>, ProtocolError> {
    if payload.len() > MAX_FRAME_LENGTH {
        return Err(ProtocolError::FrameTooLarge(payload.len()));
    }
    super::super::guard::guard(payload)?;
    let (consumed, envelope) =
        <generated::Envelope<'_> as SubRecord>::_deserialize_chained(payload)
            .map_err(ProtocolError::Decode)?;
    if consumed != payload.len() {
        return Err(ProtocolError::TrailingBytes(payload.len() - consumed));
    }
    let received = envelope.protocol_version.ok_or_else(structural_error)?;
    if received != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedProtocol {
            received,
            expected: PROTOCOL_VERSION,
        });
    }
    if envelope.body.is_none() {
        return Err(structural_error());
    }
    Ok(envelope)
}

fn node_kind(kind: u32) -> Result<generated::NodeKind, ProtocolError> {
    let kind = generated_facts::node_kind(kind).ok_or(ProtocolError::InvalidHostProperties)?;
    if matches!(kind, generated::NodeKind::Unspecified) {
        return Err(ProtocolError::InvalidHostProperties);
    }
    Ok(kind)
}

fn node_kind_value(kind: generated::NodeKind) -> Result<u32, ProtocolError> {
    if matches!(kind, generated::NodeKind::Unspecified) {
        return Err(ProtocolError::InvalidHostProperties);
    }
    Ok(u8::from(kind) as u32)
}

fn wire_transition(value: &Transition) -> generated::Transition {
    generated::Transition {
        duration_ms: Some(value.duration_ms),
        delay_ms: Some(value.delay_ms),
        easing: Some(u32::from(value.easing)),
        property_mask: Some(value.properties),
    }
}

fn decode_transition(value: generated::Transition) -> Result<Transition, ProtocolError> {
    let easing = Easing::try_from(value.easing.ok_or(ProtocolError::InvalidStyle)?)
        .map_err(|_| ProtocolError::InvalidStyle)?;
    let properties = value.property_mask.ok_or(ProtocolError::InvalidStyle)?;
    if properties == 0
        || properties
            & !(TRANSITION_OPACITY
                | TRANSITION_BACKGROUND_COLOR
                | TRANSITION_WIDTH
                | TRANSITION_HEIGHT)
            != 0
    {
        return Err(ProtocolError::InvalidStyle);
    }
    Ok(Transition {
        duration_ms: value.duration_ms.ok_or(ProtocolError::InvalidStyle)?,
        delay_ms: value.delay_ms.ok_or(ProtocolError::InvalidStyle)?,
        easing,
        properties,
    })
}

fn wire_box_shadows(value: &[BoxShadow]) -> generated::BoxShadowSet {
    generated::BoxShadowSet {
        values: Some(
            value
                .iter()
                .map(|shadow| generated::BoxShadowValue {
                    offset_x: Some(shadow.offset_x),
                    offset_y: Some(shadow.offset_y),
                    blur_radius: Some(shadow.blur_radius),
                    spread_radius: Some(shadow.spread_radius),
                    color: Some(shadow.color_rgba),
                    inset: Some(shadow.inset),
                })
                .collect(),
        ),
    }
}

fn decode_box_shadows(value: generated::BoxShadowSet) -> Result<Vec<BoxShadow>, ProtocolError> {
    let values = value.values.ok_or(ProtocolError::InvalidStyle)?;
    if values.is_empty() || values.len() > 2 {
        return Err(ProtocolError::InvalidStyle);
    }
    values
        .into_iter()
        .map(|shadow| {
            let offset_x = shadow.offset_x.ok_or(ProtocolError::InvalidStyle)?;
            let offset_y = shadow.offset_y.ok_or(ProtocolError::InvalidStyle)?;
            let blur_radius = shadow.blur_radius.ok_or(ProtocolError::InvalidStyle)?;
            let spread_radius = shadow.spread_radius.ok_or(ProtocolError::InvalidStyle)?;
            let color_rgba = shadow.color.ok_or(ProtocolError::InvalidStyle)?;
            let inset = shadow.inset.ok_or(ProtocolError::InvalidStyle)?;
            if !offset_x.is_finite()
                || !offset_y.is_finite()
                || !blur_radius.is_finite()
                || !spread_radius.is_finite()
                || blur_radius < 0.0
                || spread_radius < 0.0
            {
                return Err(ProtocolError::InvalidStyle);
            }
            Ok(BoxShadow {
                offset_x,
                offset_y,
                blur_radius,
                spread_radius,
                color_rgba,
                inset,
            })
        })
        .collect()
}
fn wire_style(value: &Style) -> Result<generated::Style<'_>, ProtocolError> {
    let wire = generated::Style {
        border_top_color: value.border_top_color,
        border_right_color: value.border_right_color,
        border_bottom_color: value.border_bottom_color,
        border_left_color: value.border_left_color,
        width: value.width,
        height: value.height,
        flex_direction: value.flex_direction.map(u32::from),
        grid_columns: value.grid_columns,
        grid_rows: value.grid_rows,
        grid_column_span: value.grid_column_span,
        grid_row_span: value.grid_row_span,
        flex_grow: value.flex_grow,
        padding: value.padding,
        gap: value.gap,
        background_color: value.background_rgba,
        color: value.color_rgba,
        opacity: value.opacity,
        transition: value.transition.as_ref().map(wire_transition),
        justify_content: value.justify_content.map(u32::from),
        align_items: value.align_items.map(u32::from),
        border_radius: value.border_radius,
        border_width: value.border_width,
        border_color: value.border_color_rgba,
        font_size: value.font_size,
        font_weight: value.font_weight.map(u32::from),
        overflow: value.overflow.map(u32::from),
        line_clamp: value.line_clamp,
        text_overflow: value.text_overflow.map(u32::from),
        margin_top: value.margin_top,
        margin_right: value.margin_right,
        margin_bottom: value.margin_bottom,
        margin_left: value.margin_left,
        font_style: value.font_style.map(u32::from),
        text_decoration: value.text_decoration.map(u32::from),
        line_height: value.line_height,
        min_width: value.min_width,
        max_width: value.max_width,
        min_height: value.min_height,
        max_height: value.max_height,
        flex_shrink: value.flex_shrink,
        align_self: value.align_self.map(u32::from),
        position: value.position.map(u32::from),
        left: value.left,
        top: value.top,
        right: value.right,
        bottom: value.bottom,
        cursor: value.cursor.map(u32::from),
        text_align: value.text_align.map(u32::from),
        box_shadow: value.box_shadows.as_deref().map(wire_box_shadows),
        linear_gradient: value.linear_gradient.map(|v| generated::LinearGradient {
            angle: Some(v.angle),
            start_color: Some(v.start_color),
            start_position: Some(v.start_position),
            end_color: Some(v.end_color),
            end_position: Some(v.end_position),
        }),
        font_family: value.font_family.as_deref(),
        padding_top: value.padding_top,
        padding_right: value.padding_right,
        padding_bottom: value.padding_bottom,
        padding_left: value.padding_left,
        border_top_width: value.border_top_width,
        border_right_width: value.border_right_width,
        border_bottom_width: value.border_bottom_width,
        border_left_width: value.border_left_width,
        border_top_left_radius: value.border_top_left_radius,
        border_top_right_radius: value.border_top_right_radius,
        border_bottom_right_radius: value.border_bottom_right_radius,
        border_bottom_left_radius: value.border_bottom_left_radius,
        width_percent: value.width_percent,
        height_percent: value.height_percent,
        flex_wrap: value.flex_wrap.map(u32::from),
    };
    validate_style_value(&wire)?;
    Ok(wire)
}

fn decode_style_code<T>(value: Option<u32>) -> Result<Option<T>, ProtocolError>
where
    T: TryFrom<u32>,
{
    value
        .map(T::try_from)
        .transpose()
        .map_err(|_| ProtocolError::InvalidStyle)
}

fn invalid_style_code<T>(value: Option<u32>) -> bool
where
    T: TryFrom<u32>,
{
    value.is_some_and(|value| T::try_from(value).is_err())
}
fn validate_style_value(value: &generated::Style<'_>) -> Result<(), ProtocolError> {
    if value.linear_gradient.as_ref().is_some_and(|v| {
        v.angle.is_none()
            || v.start_color.is_none()
            || v.start_position.is_none()
            || v.end_color.is_none()
            || v.end_position.is_none()
    }) {
        return Err(ProtocolError::InvalidStyle);
    }
    if value.linear_gradient.as_ref().is_some_and(|v| {
        !(crate::protocol::LinearGradient {
            angle: v.angle.unwrap(),
            start_color: v.start_color.unwrap(),
            start_position: v.start_position.unwrap(),
            end_color: v.end_color.unwrap(),
            end_position: v.end_position.unwrap(),
        })
        .is_valid()
    }) {
        return Err(ProtocolError::InvalidStyle);
    }
    let non_negative = [
        value.padding_top,
        value.padding_right,
        value.padding_bottom,
        value.padding_left,
        value.border_top_width,
        value.border_right_width,
        value.border_bottom_width,
        value.border_left_width,
        value.border_top_left_radius,
        value.border_top_right_radius,
        value.border_bottom_right_radius,
        value.border_bottom_left_radius,
        value.width_percent,
        value.height_percent,
        value.width,
        value.height,
        value.flex_grow,
        value.margin_top,
        value.margin_right,
        value.margin_bottom,
        value.margin_left,
        value.gap,
        value.border_radius,
        value.border_width,
        value.font_size,
        value.line_height,
        value.min_width,
        value.max_width,
        value.min_height,
        value.max_height,
        value.flex_shrink,
        value.opacity,
    ];
    if non_negative
        .into_iter()
        .flatten()
        .any(|amount| !amount.is_finite() || amount < 0.0)
        || value.width.is_some() && value.width_percent.is_some()
        || value.height.is_some() && value.height_percent.is_some()
        || invalid_style_code::<crate::protocol::FlexWrapCode>(value.flex_wrap)
        || value.opacity.is_some_and(|opacity| opacity > 1.0)
    {
        return Err(ProtocolError::InvalidStyle);
    }
    let finite_offsets = [value.left, value.top, value.right, value.bottom];
    if finite_offsets
        .into_iter()
        .flatten()
        .any(|offset| !offset.is_finite())
    {
        return Err(ProtocolError::InvalidStyle);
    }
    if [
        value.grid_columns,
        value.grid_rows,
        value.grid_column_span,
        value.grid_row_span,
    ]
    .into_iter()
    .flatten()
    .any(|count| !(1..=64).contains(&count))
        || ((value.grid_columns.is_some() || value.grid_rows.is_some())
            && value.flex_direction.is_some())
    {
        return Err(ProtocolError::InvalidStyle);
    }
    if invalid_style_code::<FlexDirectionCode>(value.flex_direction)
        || invalid_style_code::<JustifyContentCode>(value.justify_content)
        || invalid_style_code::<AlignItemsCode>(value.align_items)
        || invalid_style_code::<FontWeightCode>(value.font_weight)
        || invalid_style_code::<OverflowCode>(value.overflow)
        || value
            .line_clamp
            .is_some_and(|code| !(1..=100).contains(&code))
        || invalid_style_code::<TextOverflowCode>(value.text_overflow)
        || invalid_style_code::<FontStyleCode>(value.font_style)
        || invalid_style_code::<TextDecorationCode>(value.text_decoration)
        || invalid_style_code::<AlignSelfCode>(value.align_self)
        || invalid_style_code::<PositionCode>(value.position)
        || invalid_style_code::<CursorCode>(value.cursor)
        || invalid_style_code::<TextAlignCode>(value.text_align)
        || value.position == Some(u32::from(PositionCode::Overlay))
            && (value.right.is_some() || value.bottom.is_some())
    {
        return Err(ProtocolError::InvalidStyle);
    }
    if value.font_family.is_some_and(|family| {
        family.is_empty() || family.chars().count() > 64 || family.chars().any(char::is_control)
    }) {
        return Err(ProtocolError::InvalidStyle);
    }
    if let Some(transition) = value.transition.clone() {
        decode_transition(transition)?;
    }
    if let Some(shadows) = value.box_shadow.clone() {
        decode_box_shadows(shadows)?;
    }
    Ok(())
}

fn decode_style(value: generated::Style<'_>) -> Result<Style, ProtocolError> {
    validate_style_value(&value)?;
    Ok(Style {
        border_top_color: value.border_top_color,
        border_right_color: value.border_right_color,
        border_bottom_color: value.border_bottom_color,
        border_left_color: value.border_left_color,
        width: value.width,
        height: value.height,
        flex_direction: decode_style_code(value.flex_direction)?,
        grid_columns: value.grid_columns,
        grid_rows: value.grid_rows,
        grid_column_span: value.grid_column_span,
        grid_row_span: value.grid_row_span,
        flex_grow: value.flex_grow,
        padding: value.padding,
        gap: value.gap,
        justify_content: decode_style_code(value.justify_content)?,
        align_items: decode_style_code(value.align_items)?,
        border_radius: value.border_radius,
        border_width: value.border_width,
        border_color_rgba: value.border_color,
        font_size: value.font_size,
        font_weight: decode_style_code(value.font_weight)?,
        background_rgba: value.background_color,
        color_rgba: value.color,
        opacity: value.opacity,
        transition: value.transition.map(decode_transition).transpose()?,
        overflow: decode_style_code(value.overflow)?,
        line_clamp: value.line_clamp,
        text_overflow: decode_style_code(value.text_overflow)?,
        margin_top: value.margin_top,
        margin_right: value.margin_right,
        margin_bottom: value.margin_bottom,
        margin_left: value.margin_left,
        font_style: decode_style_code(value.font_style)?,
        text_decoration: decode_style_code(value.text_decoration)?,
        line_height: value.line_height,
        min_width: value.min_width,
        max_width: value.max_width,
        min_height: value.min_height,
        max_height: value.max_height,
        flex_shrink: value.flex_shrink,
        align_self: decode_style_code(value.align_self)?,
        position: decode_style_code(value.position)?,
        left: value.left,
        top: value.top,
        right: value.right,
        bottom: value.bottom,
        cursor: decode_style_code(value.cursor)?,
        text_align: decode_style_code(value.text_align)?,
        box_shadows: value.box_shadow.map(decode_box_shadows).transpose()?,
        linear_gradient: value
            .linear_gradient
            .map(|v| crate::protocol::LinearGradient {
                angle: v.angle.unwrap(),
                start_color: v.start_color.unwrap(),
                start_position: v.start_position.unwrap(),
                end_color: v.end_color.unwrap(),
                end_position: v.end_position.unwrap(),
            }),
        font_family: value.font_family.map(str::to_owned),
        padding_top: value.padding_top,
        padding_right: value.padding_right,
        padding_bottom: value.padding_bottom,
        padding_left: value.padding_left,
        border_top_width: value.border_top_width,
        border_right_width: value.border_right_width,
        border_bottom_width: value.border_bottom_width,
        border_left_width: value.border_left_width,
        border_top_left_radius: value.border_top_left_radius,
        border_top_right_radius: value.border_top_right_radius,
        border_bottom_right_radius: value.border_bottom_right_radius,
        border_bottom_left_radius: value.border_bottom_left_radius,
        width_percent: value.width_percent,
        height_percent: value.height_percent,
        flex_wrap: decode_style_code(value.flex_wrap)?,
    })
}

fn validate_accessibility(value: &AccessibilityProperties) -> Result<(), ProtocolError> {
    if value
        .live
        .is_some_and(|live| live > 2 || (live != 0 && (value.role <= 1 || value.value.is_none())))
    {
        return Err(ProtocolError::InvalidHostProperties);
    }
    AccessibilityRoleCode::try_from(value.role)
        .map(|_| ())
        .map_err(|_| ProtocolError::InvalidHostProperties)
}

fn validate_extension_value(value: &ExtensionValue) -> Result<(), ProtocolError> {
    match value {
        ExtensionValue::F32(value) if !value.is_finite() => {
            Err(ProtocolError::InvalidHostProperties)
        }
        ExtensionValue::Text(value) if value.len() > MAX_EXTENSION_VALUE_BYTES => {
            Err(ProtocolError::InvalidHostProperties)
        }
        ExtensionValue::Bytes(value) if value.len() > MAX_EXTENSION_VALUE_BYTES => {
            Err(ProtocolError::InvalidHostProperties)
        }
        _ => Ok(()),
    }
}

fn validate_extension_fields(fields: &[ExtensionField]) -> Result<(), ProtocolError> {
    if fields.len() > MAX_EXTENSION_FIELDS {
        return Err(ProtocolError::InvalidHostProperties);
    }
    let mut previous = None;
    let mut text_bytes = 0usize;
    let mut binary_bytes = 0usize;
    for field in fields {
        if field.id == 0 || previous.is_some_and(|previous| field.id <= previous) {
            return Err(ProtocolError::InvalidHostProperties);
        }
        validate_extension_value(&field.value)?;
        match &field.value {
            ExtensionValue::Text(value) => {
                text_bytes = text_bytes
                    .checked_add(value.len())
                    .ok_or(ProtocolError::InvalidHostProperties)?;
            }
            ExtensionValue::Bytes(value) => {
                binary_bytes = binary_bytes
                    .checked_add(value.len())
                    .ok_or(ProtocolError::InvalidHostProperties)?;
            }
            _ => {}
        }
        if text_bytes > MAX_EXTENSION_TEXT_BYTES || binary_bytes > MAX_EXTENSION_VALUE_BYTES {
            return Err(ProtocolError::InvalidHostProperties);
        }
        previous = Some(field.id);
    }
    Ok(())
}

fn validate_extension_properties(value: &ExtensionProperties) -> Result<(), ProtocolError> {
    if value.entry_id == 0
        || value.entry_version == 0
        || value.event_ids.len() > MAX_EXTENSION_EVENTS
        || value.event_ids.first() == Some(&0)
        || value.event_ids.windows(2).any(|ids| ids[0] >= ids[1])
    {
        return Err(ProtocolError::InvalidHostProperties);
    }
    validate_extension_fields(&value.fields)
}

fn wire_extension_value<'a>(value: &'a ExtensionValue) -> generated::ExtensionValue<'a> {
    match value {
        ExtensionValue::Bool(value) => generated::ExtensionValue::ExtensionBoolValue {
            value: Some(*value),
        },
        ExtensionValue::Int32(value) => generated::ExtensionValue::ExtensionInt32Value {
            value: Some(*value),
        },
        ExtensionValue::U32(value) => generated::ExtensionValue::ExtensionU32Value {
            value: Some(*value),
        },
        ExtensionValue::F32(value) => generated::ExtensionValue::ExtensionF32Value {
            value: Some(*value),
        },
        ExtensionValue::Text(value) => {
            generated::ExtensionValue::ExtensionTextValue { value: Some(value) }
        }
        ExtensionValue::Bytes(value) => generated::ExtensionValue::ExtensionBytesValue {
            value: Some(SliceWrapper::from_cooked(value)),
        },
    }
}

fn wire_extension_properties<'a>(value: &'a ExtensionProperties) -> generated::HostProperties<'a> {
    generated::HostProperties::ExtensionProperties {
        provider_id: Some(SliceWrapper::from_cooked(&value.provider_id)),
        catalog_digest: Some(SliceWrapper::from_cooked(&value.catalog_digest)),
        entry_id: Some(value.entry_id),
        entry_version: Some(value.entry_version),
        fields: Some(
            value
                .fields
                .iter()
                .map(|field| generated::ExtensionField {
                    id: Some(field.id),
                    value: Some(wire_extension_value(&field.value)),
                })
                .collect(),
        ),
        event_ids: Some(SliceWrapper::from_cooked(&value.event_ids)),
    }
}
fn wire_extension_fields<'a>(fields: &'a [ExtensionField]) -> Vec<generated::ExtensionField<'a>> {
    fields
        .iter()
        .map(|field| generated::ExtensionField {
            id: Some(field.id),
            value: Some(wire_extension_value(&field.value)),
        })
        .collect()
}

fn decode_extension_value(
    value: generated::ExtensionValue<'_>,
) -> Result<ExtensionValue, ProtocolError> {
    match value {
        generated::ExtensionValue::ExtensionBoolValue { value } => Ok(ExtensionValue::Bool(
            value.ok_or(ProtocolError::InvalidHostProperties)?,
        )),
        generated::ExtensionValue::ExtensionInt32Value { value } => Ok(ExtensionValue::Int32(
            value.ok_or(ProtocolError::InvalidHostProperties)?,
        )),
        generated::ExtensionValue::ExtensionU32Value { value } => Ok(ExtensionValue::U32(
            value.ok_or(ProtocolError::InvalidHostProperties)?,
        )),
        generated::ExtensionValue::ExtensionF32Value { value } => {
            let value = value.ok_or(ProtocolError::InvalidHostProperties)?;
            if !value.is_finite() {
                return Err(ProtocolError::InvalidHostProperties);
            }
            Ok(ExtensionValue::F32(value))
        }
        generated::ExtensionValue::ExtensionTextValue { value } => {
            let value = value.ok_or(ProtocolError::InvalidHostProperties)?;
            if value.len() > MAX_EXTENSION_VALUE_BYTES {
                return Err(ProtocolError::InvalidHostProperties);
            }
            Ok(ExtensionValue::Text(value.to_owned()))
        }
        generated::ExtensionValue::ExtensionBytesValue { value } => {
            let value = value.ok_or(ProtocolError::InvalidHostProperties)?;
            if value.len() > MAX_EXTENSION_VALUE_BYTES {
                return Err(ProtocolError::InvalidHostProperties);
            }
            Ok(ExtensionValue::Bytes(value.as_ref().to_vec()))
        }
        generated::ExtensionValue::Unknown => Err(ProtocolError::InvalidHostProperties),
    }
}

fn decode_extension_fields(
    values: Option<Vec<generated::ExtensionField<'_>>>,
) -> Result<Vec<ExtensionField>, ProtocolError> {
    let values = values.ok_or(ProtocolError::InvalidHostProperties)?;
    if values.len() > MAX_EXTENSION_FIELDS {
        return Err(ProtocolError::InvalidHostProperties);
    }
    let mut previous = None;
    let mut text_bytes = 0usize;
    let mut binary_bytes = 0usize;
    let mut fields = Vec::with_capacity(values.len());
    for field in values {
        let id = field.id.ok_or(ProtocolError::InvalidHostProperties)?;
        if id == 0 || previous.is_some_and(|previous| id <= previous) {
            return Err(ProtocolError::InvalidHostProperties);
        }
        let value =
            decode_extension_value(field.value.ok_or(ProtocolError::InvalidHostProperties)?)?;
        match &value {
            ExtensionValue::Text(value) => {
                text_bytes = text_bytes
                    .checked_add(value.len())
                    .ok_or(ProtocolError::InvalidHostProperties)?
            }
            ExtensionValue::Bytes(value) => {
                binary_bytes = binary_bytes
                    .checked_add(value.len())
                    .ok_or(ProtocolError::InvalidHostProperties)?
            }
            _ => {}
        }
        if text_bytes > MAX_EXTENSION_TEXT_BYTES || binary_bytes > MAX_EXTENSION_VALUE_BYTES {
            return Err(ProtocolError::InvalidHostProperties);
        }
        previous = Some(id);
        fields.push(ExtensionField { id, value });
    }
    Ok(fields)
}

fn decode_extension_properties(
    provider_id: Option<SliceWrapper<'_, u8>>,
    catalog_digest: Option<SliceWrapper<'_, u8>>,
    entry_id: Option<u32>,
    entry_version: Option<u32>,
    fields: Option<Vec<generated::ExtensionField<'_>>>,
    event_ids: Option<SliceWrapper<'_, u32>>,
) -> Result<ExtensionProperties, ProtocolError> {
    let provider_id = provider_id.ok_or(ProtocolError::InvalidHostProperties)?;
    let catalog_digest = catalog_digest.ok_or(ProtocolError::InvalidHostProperties)?;
    let entry_id = entry_id.ok_or(ProtocolError::InvalidHostProperties)?;
    let entry_version = entry_version.ok_or(ProtocolError::InvalidHostProperties)?;
    let event_ids = event_ids.ok_or(ProtocolError::InvalidHostProperties)?;
    if provider_id.len() != 16
        || catalog_digest.len() != 32
        || entry_id == 0
        || entry_version == 0
        || event_ids.len() > MAX_EXTENSION_EVENTS
        || event_ids.iter().next().is_some_and(|id| id == 0)
        || event_ids
            .iter()
            .zip(event_ids.iter().skip(1))
            .any(|(first, second)| first >= second)
    {
        return Err(ProtocolError::InvalidHostProperties);
    }
    let provider_id: [u8; 16] = provider_id
        .as_ref()
        .try_into()
        .map_err(|_| ProtocolError::InvalidHostProperties)?;
    let catalog_digest: [u8; 32] = catalog_digest
        .as_ref()
        .try_into()
        .map_err(|_| ProtocolError::InvalidHostProperties)?;
    Ok(ExtensionProperties {
        provider_id,
        catalog_digest,
        entry_id,
        entry_version,
        fields: decode_extension_fields(fields)?,
        event_ids: event_ids.into_iter().collect::<Arc<[u32]>>(),
    })
}

fn validate_host(value: &HostProperties) -> Result<(), ProtocolError> {
    match value {
        HostProperties::Image(image) => ObjectFitCode::try_from(image.object_fit)
            .map(|_| ())
            .map_err(|_| ProtocolError::InvalidHostProperties),
        HostProperties::Extension(extension) => validate_extension_properties(extension),
        HostProperties::Icon(icon) => {
            if !valid_host_string(&icon.name, 128)
                || !crate::icons::is_registered(&icon.name)
                || !icon.size.is_finite()
                || icon.size <= 0.0
            {
                return Err(ProtocolError::InvalidHostProperties);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
fn wire_accessibility(value: &AccessibilityProperties) -> generated::AccessibilityProperties<'_> {
    generated::AccessibilityProperties {
        role: Some(value.role),
        label: value.label.as_deref(),
        description: value.description.as_deref(),
        disabled: Some(value.disabled),
        checked: value.checked,
        selected: value.selected,
        value: value.value.as_deref(),
        expanded: value.expanded,
        level: value.level,
        live: value.live,
    }
}

fn decode_accessibility(
    value: generated::AccessibilityProperties<'_>,
) -> Result<AccessibilityProperties, ProtocolError> {
    let role = value.role.ok_or(ProtocolError::InvalidHostProperties)?;
    AccessibilityRoleCode::try_from(role).map_err(|_| ProtocolError::InvalidHostProperties)?;
    let result = AccessibilityProperties {
        role,
        label: value.label.map(str::to_owned),
        description: value.description.map(str::to_owned),
        disabled: value.disabled.ok_or(ProtocolError::InvalidHostProperties)?,
        checked: value.checked,
        selected: value.selected,
        value: value.value.map(str::to_owned),
        expanded: value.expanded,
        level: value.level,
        live: value.live,
    };
    validate_accessibility(&result)?;
    Ok(result)
}

fn wire_host(value: &HostProperties) -> generated::HostProperties<'_> {
    match value {
        HostProperties::TextInput(input) => generated::HostProperties::TextInputProperties {
            value: Some(&input.value),
            placeholder: input.placeholder.as_deref(),
            multiline: Some(input.multiline),
            disabled: Some(input.disabled),
            controlled: Some(input.controlled),
            ack_edit_seq: Some(input.ack_edit_seq),
            selection_start: Some(input.selection_start),
            selection_end: Some(input.selection_end),
            marked_start: input.marked_start,
            marked_end: input.marked_end,
            max_length: input.max_length,
            selection_reversed: Some(input.selection_reversed),
        },
        HostProperties::VirtualList(list) => generated::HostProperties::VirtualListProperties {
            item_count: Some(list.item_count),
            range_start: Some(list.range_start),
            range_end: Some(list.range_end),
            estimated_item_size: Some(list.estimated_item_size),
            overscan: Some(list.overscan),
        },
        HostProperties::Image(image) => generated::HostProperties::ImageProperties {
            source: Some(&image.source),
            object_fit: Some(image.object_fit),
            fallback_source: image.fallback_source.as_deref(),
        },
        HostProperties::Drag(drag) => generated::HostProperties::DragProperties {
            drag_type: drag.drag_type.as_deref(),
            export_files: drag
                .export_files
                .as_ref()
                .map(|files| files.iter().map(String::as_str).collect()),
            accepts_drag_over: Some(drag.accepts_drag_over),
            accepts_drop: Some(drag.accepts_drop),
        },
        HostProperties::Extension(extension) => wire_extension_properties(extension),
        HostProperties::Icon(icon) => generated::HostProperties::IconProperties {
            name: Some(&icon.name),
            size: Some(icon.size),
            color: icon.color_rgba,
        },
    }
}

fn valid_host_string(value: &str, max_len: usize) -> bool {
    !value.is_empty() && value.len() <= max_len && !value.chars().any(char::is_control)
}

fn decode_host(value: generated::HostProperties<'_>) -> Result<HostProperties, ProtocolError> {
    match value {
        generated::HostProperties::TextInputProperties {
            value,
            placeholder,
            multiline,
            disabled,
            controlled,
            ack_edit_seq,
            selection_start,
            selection_end,
            marked_start,
            marked_end,
            max_length,
            selection_reversed,
        } => {
            let marked = marked_start.zip(marked_end);
            let value = value.ok_or(ProtocolError::InvalidHostProperties)?;
            let selection_start = selection_start.ok_or(ProtocolError::InvalidHostProperties)?;
            let selection_end = selection_end.ok_or(ProtocolError::InvalidHostProperties)?;
            if value.len() > MAX_CLIPBOARD_TEXT_BYTES
                || selection_start > selection_end
                || marked_start.is_some() != marked_end.is_some()
                || marked.is_some_and(|(start, end)| start > end)
            {
                return Err(ProtocolError::InvalidHostProperties);
            }
            if placeholder.is_some_and(|value| {
                value.len() > MAX_CLIPBOARD_TEXT_BYTES || value.chars().any(char::is_control)
            }) {
                return Err(ProtocolError::InvalidHostProperties);
            }
            let placeholder = placeholder.map(str::to_owned);
            Ok(HostProperties::TextInput(TextInputProperties {
                value: value.to_owned(),
                placeholder,
                multiline: multiline.ok_or(ProtocolError::InvalidHostProperties)?,
                disabled: disabled.ok_or(ProtocolError::InvalidHostProperties)?,
                controlled: controlled.ok_or(ProtocolError::InvalidHostProperties)?,
                ack_edit_seq: ack_edit_seq.ok_or(ProtocolError::InvalidHostProperties)?,
                selection_start,
                selection_end,
                marked_start: marked.map(|(start, _)| start),
                marked_end: marked.map(|(_, end)| end),
                max_length,
                selection_reversed: selection_reversed
                    .ok_or(ProtocolError::InvalidHostProperties)?,
            }))
        }
        generated::HostProperties::VirtualListProperties {
            item_count,
            range_start,
            range_end,
            estimated_item_size,
            overscan,
        } => {
            let item_count = item_count.ok_or(ProtocolError::InvalidHostProperties)?;
            let range_start = range_start.ok_or(ProtocolError::InvalidHostProperties)?;
            let range_end = range_end.ok_or(ProtocolError::InvalidHostProperties)?;
            let estimated_item_size =
                estimated_item_size.ok_or(ProtocolError::InvalidHostProperties)?;
            if range_start > range_end
                || range_end > item_count
                || !estimated_item_size.is_finite()
                || estimated_item_size <= 0.0
            {
                return Err(ProtocolError::InvalidHostProperties);
            }
            Ok(HostProperties::VirtualList(VirtualListProperties {
                item_count,
                range_start,
                range_end,
                estimated_item_size,
                overscan: overscan.ok_or(ProtocolError::InvalidHostProperties)?,
            }))
        }
        generated::HostProperties::ImageProperties {
            source,
            object_fit,
            fallback_source,
        } => {
            let source = source.ok_or(ProtocolError::InvalidHostProperties)?;
            let object_fit = object_fit.ok_or(ProtocolError::InvalidHostProperties)?;
            ObjectFitCode::try_from(object_fit)
                .map_err(|_| ProtocolError::InvalidHostProperties)?;
            if !valid_host_string(source, super::super::MAX_IMAGE_SOURCE_BYTES)
                || fallback_source.is_some_and(|value| {
                    !valid_host_string(value, super::super::MAX_IMAGE_SOURCE_BYTES)
                })
            {
                return Err(ProtocolError::InvalidHostProperties);
            }
            Ok(HostProperties::Image(ImageProperties {
                source: source.to_owned(),
                object_fit,
                fallback_source: fallback_source.map(str::to_owned),
            }))
        }
        generated::HostProperties::DragProperties {
            drag_type,
            export_files,
            accepts_drag_over,
            accepts_drop,
        } => {
            let drag_type = drag_type.map(str::to_owned);
            if drag_type
                .as_deref()
                .is_some_and(|value| !valid_host_string(value, 128))
            {
                return Err(ProtocolError::InvalidHostProperties);
            }
            let export_files =
                export_files.map(|files| files.into_iter().map(str::to_owned).collect::<Vec<_>>());
            if export_files.as_ref().is_some_and(|files| {
                files.len() > 256 || files.iter().any(|value| !valid_host_string(value, 4096))
            }) {
                return Err(ProtocolError::InvalidHostProperties);
            }
            Ok(HostProperties::Drag(DragProperties {
                drag_type,
                export_files,
                accepts_drag_over: accepts_drag_over.ok_or(ProtocolError::InvalidHostProperties)?,
                accepts_drop: accepts_drop.ok_or(ProtocolError::InvalidHostProperties)?,
            }))
        }
        generated::HostProperties::ExtensionProperties {
            provider_id,
            catalog_digest,
            entry_id,
            entry_version,
            fields,
            event_ids,
        } => Ok(HostProperties::Extension(decode_extension_properties(
            provider_id,
            catalog_digest,
            entry_id,
            entry_version,
            fields,
            event_ids,
        )?)),
        generated::HostProperties::IconProperties { name, size, color } => {
            let name = name.ok_or(ProtocolError::InvalidHostProperties)?;
            let size = size.ok_or(ProtocolError::InvalidHostProperties)?;
            if !valid_host_string(name, 128)
                || !crate::icons::is_registered(name)
                || !size.is_finite()
                || size <= 0.0
            {
                return Err(ProtocolError::InvalidHostProperties);
            }
            Ok(HostProperties::Icon(IconProperties {
                name: name.to_owned(),
                size,
                color_rgba: color,
            }))
        }
        generated::HostProperties::Unknown => Err(ProtocolError::InvalidHostProperties),
    }
}

fn wire_node(value: &Node) -> Result<generated::Node<'_>, ProtocolError> {
    let path = |field: &str| format!("node {}.{field}", value.id);
    Ok(generated::Node {
        id: Some(value.id),
        parent_id: Some(value.parent_id),
        index: Some(value.index),
        kind: Some(node_kind(value.kind).map_err(|error| error.at(path("kind")))?),
        style: value
            .style
            .as_ref()
            .map(|style| wire_style(style).map_err(|error| error.at(path("style"))))
            .transpose()?,
        text: value.text.as_deref(),
        listener_id: Some(value.listener_id),
        host_properties: value
            .host_properties
            .as_ref()
            .map(|host| {
                validate_host(host).map_err(|error| error.at(path("hostProperties")))?;
                Ok(wire_host(host))
            })
            .transpose()?,
        accessibility: value
            .accessibility
            .as_ref()
            .map(|accessibility| {
                validate_accessibility(accessibility)
                    .map_err(|error| error.at(path("accessibility")))?;
                Ok(wire_accessibility(accessibility))
            })
            .transpose()?,
        focusable: Some(value.focusable),
        selectable: Some(value.selectable),
        tooltip: value.tooltip.as_deref(),
        accepts_pointer_move: Some(value.accepts_pointer_move),
    })
}

fn decode_node(value: generated::Node<'_>) -> Result<Node, ProtocolError> {
    let id = value.id.ok_or(ProtocolError::InvalidHostProperties)?;
    let path = |field: &str| format!("node {id}.{field}");
    let parent_id = value
        .parent_id
        .ok_or(ProtocolError::InvalidHostProperties)
        .map_err(|error| error.at(path("parentId")))?;
    let index = value
        .index
        .ok_or(ProtocolError::InvalidHostProperties)
        .map_err(|error| error.at(path("index")))?;
    let kind = node_kind_value(
        value
            .kind
            .ok_or(ProtocolError::InvalidHostProperties)
            .map_err(|error| error.at(path("kind")))?,
    )
    .map_err(|error| error.at(path("kind")))?;
    let style = value
        .style
        .map(|style| decode_style(style).map_err(|error| error.at(path("style"))))
        .transpose()?;
    let listener_id = value
        .listener_id
        .ok_or(ProtocolError::InvalidHostProperties)
        .map_err(|error| error.at(path("listenerId")))?;
    let host_properties = value
        .host_properties
        .map(|host| decode_host(host).map_err(|error| error.at(path("hostProperties"))))
        .transpose()?;
    let accessibility = value
        .accessibility
        .map(|accessibility| {
            decode_accessibility(accessibility).map_err(|error| error.at(path("accessibility")))
        })
        .transpose()?;
    let focusable = value
        .focusable
        .ok_or(ProtocolError::InvalidHostProperties)
        .map_err(|error| error.at(path("focusable")))?;
    let selectable = value
        .selectable
        .ok_or(ProtocolError::InvalidHostProperties)
        .map_err(|error| error.at(path("selectable")))?;
    let accepts_pointer_move = value
        .accepts_pointer_move
        .ok_or(ProtocolError::InvalidHostProperties)
        .map_err(|error| error.at(path("acceptsPointerMove")))?;
    Ok(Node {
        id,
        parent_id,
        index,
        kind,
        style,
        text: value.text.map(str::to_owned),
        listener_id,
        host_properties,
        accessibility,
        focusable,
        selectable,
        tooltip: value.tooltip.map(str::to_owned),
        accepts_pointer_move,
    })
}

fn wire_snapshot(value: &Snapshot) -> Result<generated::Body<'_>, ProtocolError> {
    Ok(generated::Body::Snapshot {
        surface_id: Some(value.surface_id),
        epoch: Some(value.epoch),
        base_revision: Some(value.base_revision),
        revision: Some(value.revision),
        nodes: Some(
            value
                .nodes
                .iter()
                .enumerate()
                .map(|(index, node)| {
                    wire_node(node)
                        .map_err(|error| error.at(format!("body.snapshot.nodes[{index}]")))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    })
}

pub(super) fn encode_snapshot(value: &Snapshot) -> Result<Vec<u8>, ProtocolError> {
    encode_envelope(wire_snapshot(value)?)
}

fn decode_snapshot_body(body: generated::Body<'_>) -> Result<Snapshot, ProtocolError> {
    match body {
        generated::Body::Snapshot {
            surface_id,
            epoch,
            base_revision,
            revision,
            nodes,
        } => Ok(Snapshot {
            surface_id: surface_id.ok_or_else(structural_error)?,
            epoch: epoch.ok_or_else(structural_error)?,
            base_revision: base_revision.ok_or_else(structural_error)?,
            revision: revision.ok_or_else(structural_error)?,
            nodes: nodes
                .ok_or_else(structural_error)?
                .into_iter()
                .enumerate()
                .map(|(index, node)| {
                    decode_node(node)
                        .map_err(|error| error.at(format!("body.snapshot.nodes[{index}]")))
                })
                .collect::<Result<Vec<_>, _>>()?,
        }),
        generated::Body::Unknown => Err(structural_error()),
        _ => Err(ProtocolError::WrongMessageType(0)),
    }
}

pub(super) fn decode_snapshot(payload: &[u8]) -> Result<Snapshot, ProtocolError> {
    decode_snapshot_body(
        decode_envelope(payload)?
            .body
            .ok_or_else(structural_error)?,
    )
}

fn wire_patch_operation(
    value: &PatchOperation,
) -> Result<generated::PatchOperation<'_>, ProtocolError> {
    let operation = match value {
        PatchOperation::Create(node) => generated::PatchOperationValue::PatchCreate {
            node: Some(wire_node(node)?),
        },
        PatchOperation::Move {
            id,
            parent_id,
            index,
        } => generated::PatchOperationValue::PatchMove {
            id: Some(*id),
            parent_id: Some(*parent_id),
            index: Some(*index),
        },
        PatchOperation::Delete { id } => {
            generated::PatchOperationValue::PatchDelete { id: Some(*id) }
        }
        PatchOperation::Update {
            id,
            mask,
            style,
            text,
            listener_id,
            host_properties,
            accessibility,
            focusable,
            selectable,
            tooltip,
            accepts_pointer_move,
        } => generated::PatchOperationValue::PatchUpdate {
            id: Some(*id),
            mask: Some(*mask),
            style: if *mask & UPDATE_STYLE != 0 {
                style
                    .as_ref()
                    .map(|style| {
                        wire_style(style)
                            .map_err(|error| error.at(format!("patch node {id}.style")))
                    })
                    .transpose()?
            } else {
                None
            },
            clear_style: if *mask & UPDATE_STYLE != 0 && style.is_none() {
                Some(generated::ClearStyle {})
            } else {
                None
            },
            text: if *mask & UPDATE_TEXT != 0 {
                text.as_deref()
            } else {
                None
            },
            listener_id: if *mask & UPDATE_LISTENER != 0 {
                Some(*listener_id)
            } else {
                None
            },
            host_properties: if *mask & UPDATE_PROPERTIES != 0 {
                host_properties
                    .as_ref()
                    .map(|host| {
                        validate_host(host)
                            .map_err(|error| error.at(format!("patch node {id}.hostProperties")))?;
                        Ok(wire_host(host))
                    })
                    .transpose()?
            } else {
                None
            },
            accessibility: if *mask & UPDATE_ACCESSIBILITY != 0 {
                accessibility
                    .as_ref()
                    .map(|accessibility| {
                        validate_accessibility(accessibility)
                            .map_err(|error| error.at(format!("patch node {id}.accessibility")))?;
                        Ok(wire_accessibility(accessibility))
                    })
                    .transpose()?
            } else {
                None
            },
            focusable: if *mask & UPDATE_FOCUSABLE != 0 {
                Some(*focusable)
            } else {
                None
            },
            selectable: if *mask & UPDATE_SELECTABLE != 0 {
                Some(*selectable)
            } else {
                None
            },
            tooltip: if *mask & UPDATE_TOOLTIP != 0 {
                tooltip.as_deref()
            } else {
                None
            },
            accepts_pointer_move: if *mask & UPDATE_POINTER_MOVE != 0 {
                Some(*accepts_pointer_move)
            } else {
                None
            },
        },
    };
    Ok(generated::PatchOperation {
        operation: Some(operation),
    })
}

pub(super) fn encode_patch(value: &Patch) -> Result<Vec<u8>, ProtocolError> {
    encode_envelope(generated::Body::Patch {
        surface_id: Some(value.surface_id),
        epoch: Some(value.epoch),
        base_revision: Some(value.base_revision),
        revision: Some(value.revision),
        operations: Some(
            value
                .operations
                .iter()
                .enumerate()
                .map(|(index, operation)| {
                    wire_patch_operation(operation)
                        .map_err(|error| error.at(format!("body.patch.operations[{index}]")))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    })
}

fn decode_patch_body(body: generated::Body<'_>) -> Result<Patch, ProtocolError> {
    match body {
        generated::Body::Patch {
            surface_id,
            epoch,
            base_revision,
            revision,
            operations,
        } => Ok(Patch {
            surface_id: surface_id.ok_or_else(structural_error)?,
            epoch: epoch.ok_or_else(structural_error)?,
            base_revision: base_revision.ok_or_else(structural_error)?,
            revision: revision.ok_or_else(structural_error)?,
            operations: operations
                .ok_or_else(structural_error)?
                .into_iter()
                .enumerate()
                .map(|(index, operation)| {
                    decode_patch_operation(operation)
                        .map_err(|error| error.at(format!("body.patch.operations[{index}]")))
                })
                .collect::<Result<Vec<_>, _>>()?,
        }),
        generated::Body::Unknown => Err(structural_error()),
        _ => Err(ProtocolError::WrongMessageType(0)),
    }
}

pub(super) fn decode_patch(payload: &[u8]) -> Result<Patch, ProtocolError> {
    decode_patch_body(
        decode_envelope(payload)?
            .body
            .ok_or_else(structural_error)?,
    )
}

fn decode_patch_operation(
    value: generated::PatchOperation<'_>,
) -> Result<PatchOperation, ProtocolError> {
    match value.operation.ok_or_else(structural_error)? {
        generated::PatchOperationValue::PatchCreate { node } => Ok(PatchOperation::Create(
            decode_node(node.ok_or_else(structural_error)?)?,
        )),
        generated::PatchOperationValue::PatchMove {
            id,
            parent_id,
            index,
        } => Ok(PatchOperation::Move {
            id: id.ok_or_else(structural_error)?,
            parent_id: parent_id.ok_or_else(structural_error)?,
            index: index.ok_or_else(structural_error)?,
        }),
        generated::PatchOperationValue::PatchDelete { id } => Ok(PatchOperation::Delete {
            id: id.ok_or_else(structural_error)?,
        }),
        generated::PatchOperationValue::PatchUpdate {
            id,
            mask,
            style,
            clear_style,
            text,
            listener_id,
            host_properties,
            accessibility,
            focusable,
            selectable,
            tooltip,
            accepts_pointer_move,
        } => {
            let id = id.ok_or_else(structural_error)?;
            let path = |field: &str| format!("patch node {id}.{field}");
            let invalid = |field: &str| ProtocolError::InvalidCommandPayload.at(path(field));
            let mask = mask.ok_or_else(structural_error)?;
            if mask
                & !(UPDATE_STYLE
                    | UPDATE_TEXT
                    | UPDATE_LISTENER
                    | UPDATE_PROPERTIES
                    | UPDATE_ACCESSIBILITY
                    | UPDATE_FOCUSABLE
                    | UPDATE_SELECTABLE
                    | UPDATE_TOOLTIP
                    | UPDATE_POINTER_MOVE)
                != 0
            {
                return Err(invalid("mask"));
            }
            if style.is_some() && clear_style.is_some()
                || mask & UPDATE_STYLE == 0 && (style.is_some() || clear_style.is_some())
                || mask & UPDATE_STYLE != 0 && style.is_none() && clear_style.is_none()
            {
                return Err(ProtocolError::InvalidStyle.at(path("style")));
            }
            if mask & UPDATE_TEXT == 0 && text.is_some() {
                return Err(invalid("text"));
            }
            if mask & UPDATE_LISTENER != 0 && listener_id.is_none()
                || mask & UPDATE_LISTENER == 0 && listener_id.is_some()
            {
                return Err(invalid("listenerId"));
            }
            if mask & UPDATE_PROPERTIES == 0 && host_properties.is_some() {
                return Err(invalid("hostProperties"));
            }
            if mask & UPDATE_ACCESSIBILITY == 0 && accessibility.is_some() {
                return Err(invalid("accessibility"));
            }
            if mask & UPDATE_FOCUSABLE != 0 && focusable.is_none()
                || mask & UPDATE_FOCUSABLE == 0 && focusable.is_some()
            {
                return Err(invalid("focusable"));
            }
            if mask & UPDATE_SELECTABLE != 0 && selectable.is_none()
                || mask & UPDATE_SELECTABLE == 0 && selectable.is_some()
            {
                return Err(invalid("selectable"));
            }
            if mask & UPDATE_TOOLTIP == 0 && tooltip.is_some() {
                return Err(invalid("tooltip"));
            }
            if mask & UPDATE_POINTER_MOVE != 0 && accepts_pointer_move.is_none()
                || mask & UPDATE_POINTER_MOVE == 0 && accepts_pointer_move.is_some()
            {
                return Err(invalid("acceptsPointerMove"));
            }
            Ok(PatchOperation::Update {
                id,
                mask,
                style: style
                    .map(|style| decode_style(style).map_err(|error| error.at(path("style"))))
                    .transpose()?,
                text: text.map(str::to_owned),
                listener_id: listener_id.unwrap_or(0),
                host_properties: host_properties
                    .map(|host| decode_host(host).map_err(|error| error.at(path("hostProperties"))))
                    .transpose()?,
                accessibility: accessibility
                    .map(|accessibility| {
                        decode_accessibility(accessibility)
                            .map_err(|error| error.at(path("accessibility")))
                    })
                    .transpose()?,
                focusable: focusable.unwrap_or(false),
                selectable: selectable.unwrap_or(false),
                tooltip: tooltip.map(str::to_owned),
                accepts_pointer_move: accepts_pointer_move.unwrap_or(false),
            })
        }
        generated::PatchOperationValue::Unknown => Err(ProtocolError::UnknownPatchOperation(0)),
    }
}
fn wire_window_options(value: &WindowOpenOptions) -> generated::WindowOpenOptions {
    let (min_width, min_height) = value
        .min_size
        .map_or((None, None), |(width, height)| (Some(width), Some(height)));
    generated::WindowOpenOptions {
        kind: value.kind,
        resizable: value.resizable,
        min_width,
        min_height,
    }
}

fn decode_window_options(
    value: generated::WindowOpenOptions,
) -> Result<WindowOpenOptions, ProtocolError> {
    if value.min_width.is_some() != value.min_height.is_some()
        || value.kind.is_some_and(|kind| kind > 2)
    {
        return Err(ProtocolError::InvalidCommandPayload);
    }
    Ok(WindowOpenOptions {
        kind: value.kind,
        resizable: value.resizable,
        min_size: value.min_width.zip(value.min_height),
    })
}

fn wire_notification_actions<'a>(
    value: &'a [NotificationActionDefinition],
) -> Vec<generated::NotificationActionDefinition<'a>> {
    value
        .iter()
        .map(|action| generated::NotificationActionDefinition {
            id: Some(&action.id),
            label: Some(&action.label),
        })
        .collect()
}

fn decode_notification_actions(
    value: Vec<generated::NotificationActionDefinition<'_>>,
) -> Result<Vec<NotificationActionDefinition>, ProtocolError> {
    value
        .into_iter()
        .map(|action| {
            Ok(NotificationActionDefinition {
                id: action
                    .id
                    .ok_or(ProtocolError::InvalidCommandPayload)?
                    .to_owned(),
                label: action
                    .label
                    .ok_or(ProtocolError::InvalidCommandPayload)?
                    .to_owned(),
            })
        })
        .collect()
}

fn wire_menu<'a>(value: &'a MenuDefinition) -> generated::MenuDefinition<'a> {
    generated::MenuDefinition {
        title: Some(&value.title),
        items: Some(value.items.iter().map(wire_menu_item).collect()),
    }
}

fn wire_menu_item<'a>(value: &'a MenuItemDefinition) -> generated::MenuItem<'a> {
    let item = match value {
        MenuItemDefinition::Separator => generated::MenuItemValue::MenuSeparator {},
        MenuItemDefinition::Action {
            name,
            disabled,
            checked,
        } => generated::MenuItemValue::MenuAction {
            name: Some(name),
            disabled: Some(*disabled),
            checked: Some(*checked),
        },
        MenuItemDefinition::Submenu(menu) => generated::MenuItemValue::MenuSubmenu {
            menu: Some(wire_menu(menu)),
        },
    };
    generated::MenuItem { value: Some(item) }
}

fn wire_keybindings<'a>(
    value: &'a [KeybindingDefinition],
) -> Vec<generated::KeybindingDefinition<'a>> {
    value
        .iter()
        .map(|binding| generated::KeybindingDefinition {
            keystrokes: Some(&binding.keystrokes),
            action_name: Some(&binding.action_name),
        })
        .collect()
}

fn decode_keybindings(
    value: Vec<generated::KeybindingDefinition<'_>>,
) -> Result<Vec<KeybindingDefinition>, ProtocolError> {
    value
        .into_iter()
        .map(|binding| {
            Ok(KeybindingDefinition {
                keystrokes: binding
                    .keystrokes
                    .ok_or(ProtocolError::InvalidCommandPayload)?
                    .to_owned(),
                action_name: binding
                    .action_name
                    .ok_or(ProtocolError::InvalidCommandPayload)?
                    .to_owned(),
            })
        })
        .collect()
}

fn decode_image(
    format: Option<u32>,
    bytes: Option<SliceWrapper<'_, u8>>,
) -> Result<ClipboardImage, ProtocolError> {
    let format = format.ok_or(ProtocolError::InvalidCommandPayload)?;
    let bytes = bytes.ok_or(ProtocolError::InvalidCommandPayload)?;
    if ClipboardImageFormatCode::try_from(format).is_err()
        || bytes.as_ref().is_empty()
        || bytes.as_ref().len() > MAX_CLIPBOARD_IMAGE_BYTES
    {
        return Err(ProtocolError::InvalidCommandPayload);
    }
    Ok(ClipboardImage {
        format,
        bytes: bytes.as_ref().to_vec(),
    })
}

fn wire_command_value<'a>(value: &'a CommandValue) -> generated::CommandValue<'a> {
    match value {
        CommandValue::Number(value) => generated::CommandValue::NumberValue {
            value: Some(*value),
        },
        CommandValue::Pair((width, height)) => generated::CommandValue::PairValue {
            width: Some(*width),
            height: Some(*height),
        },
        CommandValue::Bool(value) => generated::CommandValue::BoolValue {
            value: Some(*value),
        },
        CommandValue::Text(value) => generated::CommandValue::TextValue { value: Some(value) },
        CommandValue::Paths(value) => generated::CommandValue::PathsValue {
            paths: Some(value.iter().map(String::as_str).collect()),
        },
        CommandValue::FileText(value) => {
            generated::CommandValue::FileTextValue { value: Some(value) }
        }
        CommandValue::Image(value) => generated::CommandValue::ImageValue {
            format: Some(value.format),
            bytes: Some(SliceWrapper::from_cooked(&value.bytes)),
        },
        CommandValue::Bounds((x, y, width, height)) => generated::CommandValue::BoundsValue {
            x: Some(*x),
            y: Some(*y),
            width: Some(*width),
            height: Some(*height),
        },
        CommandValue::WindowState((fullscreen, maximized)) => {
            generated::CommandValue::WindowStateValue {
                fullscreen: Some(*fullscreen),
                maximized: Some(*maximized),
            }
        }
        CommandValue::ScrollOffset(value) => generated::CommandValue::ScrollOffsetValue {
            value: Some(*value),
        },
        CommandValue::Bytes(value) => generated::CommandValue::BytesValue {
            value: Some(SliceWrapper::from_cooked(value)),
        },
    }
}

fn decode_command_value(value: generated::CommandValue<'_>) -> Result<CommandValue, ProtocolError> {
    match value {
        generated::CommandValue::NumberValue { value } => Ok(CommandValue::Number(
            value.ok_or(ProtocolError::InvalidCommandPayload)?,
        )),
        generated::CommandValue::PairValue { width, height } => Ok(CommandValue::Pair((
            width.ok_or(ProtocolError::InvalidCommandPayload)?,
            height.ok_or(ProtocolError::InvalidCommandPayload)?,
        ))),
        generated::CommandValue::BoolValue { value } => Ok(CommandValue::Bool(
            value.ok_or(ProtocolError::InvalidCommandPayload)?,
        )),
        generated::CommandValue::TextValue { value } => Ok(CommandValue::Text(
            value
                .ok_or(ProtocolError::InvalidCommandPayload)?
                .to_owned(),
        )),
        generated::CommandValue::PathsValue { paths } => Ok(CommandValue::Paths(
            paths
                .ok_or(ProtocolError::InvalidCommandPayload)?
                .into_iter()
                .map(str::to_owned)
                .collect(),
        )),
        generated::CommandValue::FileTextValue { value } => Ok(CommandValue::FileText(
            value
                .ok_or(ProtocolError::InvalidCommandPayload)?
                .to_owned(),
        )),
        generated::CommandValue::ImageValue { format, bytes } => {
            Ok(CommandValue::Image(decode_image(format, bytes)?))
        }
        generated::CommandValue::BoundsValue {
            x,
            y,
            width,
            height,
        } => Ok(CommandValue::Bounds((
            x.ok_or(ProtocolError::InvalidCommandPayload)?,
            y.ok_or(ProtocolError::InvalidCommandPayload)?,
            width.ok_or(ProtocolError::InvalidCommandPayload)?,
            height.ok_or(ProtocolError::InvalidCommandPayload)?,
        ))),
        generated::CommandValue::WindowStateValue {
            fullscreen,
            maximized,
        } => Ok(CommandValue::WindowState((
            fullscreen.ok_or(ProtocolError::InvalidCommandPayload)?,
            maximized.ok_or(ProtocolError::InvalidCommandPayload)?,
        ))),
        generated::CommandValue::ScrollOffsetValue { value } => Ok(CommandValue::ScrollOffset(
            value.ok_or(ProtocolError::InvalidCommandPayload)?,
        )),
        generated::CommandValue::BytesValue { value } => {
            let value = value.ok_or(ProtocolError::InvalidCommandPayload)?;
            if value.len() > MAX_NATIVE_CALL_BYTES {
                return Err(ProtocolError::InvalidCommandPayload);
            }
            Ok(CommandValue::Bytes(value.as_ref().to_vec()))
        }
        generated::CommandValue::Unknown => Err(ProtocolError::InvalidCommandPayload),
    }
}

fn wire_command_payload<'a>(
    operation: &'a CommandOperation,
) -> Option<generated::CommandPayload<'a>> {
    match operation {
        CommandOperation::OpenPopup {
            anchor_node_id,
            width,
            height,
            placement,
            gap,
        } => Some(generated::CommandPayload::OpenPopupCommand {
            anchor_node_id: Some(*anchor_node_id),
            width: Some(*width),
            height: Some(*height),
            placement: Some(*placement),
            gap: Some(*gap),
        }),
        CommandOperation::ClosePopup { request_id } => {
            Some(generated::CommandPayload::ClosePopupCommand {
                request_id: Some(*request_id),
            })
        }
        CommandOperation::ConfigureApplication {
            keep_alive,
            quit,
            acknowledged_sequence,
        } => Some(generated::CommandPayload::ConfigureApplicationCommand {
            keep_alive: Some(*keep_alive),
            quit: Some(*quit),
            acknowledged_sequence: Some(*acknowledged_sequence),
        }),
        CommandOperation::CancelNative { request_id } => {
            Some(generated::CommandPayload::CancelNativeCommand {
                request_id: Some(*request_id),
            })
        }
        CommandOperation::InvokeNative {
            module_id,
            module_digest,
            function_id,
            args,
        } => Some(generated::CommandPayload::InvokeNativeCommand {
            module_id: Some(SliceWrapper::from_cooked(module_id)),
            module_digest: Some(SliceWrapper::from_cooked(module_digest)),
            function_id: Some(*function_id),
            args: Some(SliceWrapper::from_cooked(args)),
        }),
        CommandOperation::Focus
        | CommandOperation::Blur
        | CommandOperation::ScrollToEnd
        | CommandOperation::ZoomWindow
        | CommandOperation::ToggleFullscreen
        | CommandOperation::FocusNext
        | CommandOperation::FocusPrev
        | CommandOperation::GetWindowSize
        | CommandOperation::GetFocus
        | CommandOperation::ClipboardRead
        | CommandOperation::ClipboardReadImage
        | CommandOperation::MinimizeWindow
        | CommandOperation::GetWindowBounds
        | CommandOperation::GetWindowState
        | CommandOperation::ActivateWindow
        | CommandOperation::GetScrollOffset => None,
        CommandOperation::SetSelection { start, end }
        | CommandOperation::ScrollToIndex {
            index: start,
            alignment: end,
        }
        | CommandOperation::ResizeWindow {
            width: start,
            height: end,
        } => Some(generated::CommandPayload::U32PairCommand {
            first: Some(*start),
            second: Some(*end),
        }),
        CommandOperation::ResolveCloseRequest { request_id, allow } => {
            Some(generated::CommandPayload::CloseResolutionCommand {
                request_id: Some(*request_id),
                allow: Some(*allow),
            })
        }
        CommandOperation::ScrollToOffset { offset } => {
            Some(generated::CommandPayload::FloatCommand {
                value: Some(*offset),
            })
        }
        CommandOperation::OpenSurface {
            title,
            width,
            height,
            options,
        } => Some(generated::CommandPayload::OpenSurfaceCommand {
            title: Some(title),
            width: Some(*width),
            height: Some(*height),
            options: options.as_ref().map(wire_window_options),
        }),
        CommandOperation::FileDialogOpen {
            title,
            directories,
            multiple,
        } => Some(generated::CommandPayload::FileDialogOpenCommand {
            title: Some(title),
            directories: Some(*directories),
            multiple: Some(*multiple),
        }),
        CommandOperation::ShowNotification {
            title,
            body,
            actions,
        } => Some(generated::CommandPayload::NotificationCommand {
            title: Some(title),
            body: Some(body),
            actions: actions.as_deref().map(wire_notification_actions),
        }),
        CommandOperation::SetMenus { menus } => Some(generated::CommandPayload::MenusCommand {
            menus: Some(menus.iter().map(wire_menu).collect()),
        }),
        CommandOperation::SetKeybindings { bindings } => {
            Some(generated::CommandPayload::KeybindingsCommand {
                bindings: Some(wire_keybindings(bindings)),
            })
        }
        CommandOperation::ClipboardWriteImage { image } => {
            Some(generated::CommandPayload::ClipboardImageCommand {
                format: Some(image.format),
                bytes: Some(SliceWrapper::from_cooked(&image.bytes)),
            })
        }
        CommandOperation::WriteTextFile { path, content } => {
            Some(generated::CommandPayload::StringPairCommand {
                path: Some(path),
                content: Some(content),
            })
        }
        CommandOperation::SetTitle { title }
        | CommandOperation::FileDialogSave {
            default_name: title,
        }
        | CommandOperation::ClipboardWrite { text: title }
        | CommandOperation::SetClosePolicy { policy: title }
        | CommandOperation::ReadTextFile { path: title }
        | CommandOperation::LoadFont { path: title }
        | CommandOperation::OpenUrl { url: title } => {
            Some(generated::CommandPayload::TextCommand { value: Some(title) })
        }
    }
}

pub(super) fn encode_command(value: &Command) -> Result<Vec<u8>, ProtocolError> {
    validate_command(value)?;
    let kind = value.operation.kind();
    encode_envelope(generated::Body::Command {
        surface_id: Some(value.meta.surface_id),
        epoch: Some(value.meta.epoch),
        after_revision: Some(value.meta.after_revision),
        request_id: Some(value.meta.request_id),
        node_id: Some(value.meta.node_id),
        kind: Some(wire_command_kind(kind.into())?),
        payload: wire_command_payload(&value.operation),
    })
}

fn validate_command(value: &Command) -> Result<(), ProtocolError> {
    let kind = value.operation.kind();
    let path = |field: &str| format!("body.command({kind:?}).{field}");
    let invalid = |field: &str| ProtocolError::InvalidCommandPayload.at(path(field));
    if matches!(
        value.operation,
        CommandOperation::SetTitle { .. }
            | CommandOperation::ResizeWindow { .. }
            | CommandOperation::ZoomWindow
            | CommandOperation::ToggleFullscreen
            | CommandOperation::OpenUrl { .. }
            | CommandOperation::FocusNext
            | CommandOperation::FocusPrev
            | CommandOperation::GetWindowSize
            | CommandOperation::ClipboardWrite { .. }
            | CommandOperation::ClipboardRead
            | CommandOperation::OpenSurface { .. }
            | CommandOperation::FileDialogOpen { .. }
            | CommandOperation::FileDialogSave { .. }
            | CommandOperation::ShowNotification { .. }
            | CommandOperation::SetMenus { .. }
            | CommandOperation::SetKeybindings { .. }
            | CommandOperation::SetClosePolicy { .. }
            | CommandOperation::ResolveCloseRequest { .. }
            | CommandOperation::ReadTextFile { .. }
            | CommandOperation::WriteTextFile { .. }
            | CommandOperation::ClipboardWriteImage { .. }
            | CommandOperation::ClipboardReadImage
            | CommandOperation::LoadFont { .. }
            | CommandOperation::MinimizeWindow
            | CommandOperation::GetWindowBounds
            | CommandOperation::GetWindowState
            | CommandOperation::ActivateWindow
    ) && value.meta.node_id != 1
    {
        return Err(invalid("nodeId"));
    }
    match &value.operation {
        CommandOperation::OpenPopup {
            anchor_node_id,
            width,
            height,
            placement,
            gap,
        } if value.meta.node_id != 1
            || *anchor_node_id < 2
            || *width == 0
            || *height == 0
            || !valid_surface_size(*width, *height)
            || *placement > 11
            || !gap.is_finite()
            || !(0.0..=1024.0).contains(gap) =>
        {
            return Err(invalid("popup options"));
        }
        CommandOperation::ClosePopup { request_id }
            if value.meta.node_id != 1 || *request_id == 0 =>
        {
            return Err(invalid("popup surface"));
        }
        CommandOperation::ConfigureApplication {
            keep_alive, quit, ..
        } if value.meta.surface_id != 0
            || value.meta.node_id != 0
            || value.meta.after_revision != 0
            || value.meta.epoch == 0
            || value.meta.request_id == 0
            || (*keep_alive && *quit) =>
        {
            return Err(invalid("application control"));
        }

        CommandOperation::CancelNative { request_id }
            if *request_id == 0 || value.meta.node_id != 1 =>
        {
            return Err(invalid("native cancellation"));
        }
        CommandOperation::InvokeNative {
            function_id, args, ..
        } => {
            if value.meta.node_id == 0 {
                return Err(invalid("nodeId"));
            }
            if *function_id == 0 {
                return Err(invalid("functionId"));
            }
            if args.len() > MAX_NATIVE_CALL_BYTES {
                return Err(invalid("args"));
            }
        }
        CommandOperation::SetSelection { start, end } if start > end => {
            return Err(invalid("selection"));
        }
        CommandOperation::ResizeWindow { width, height }
            if !valid_surface_size(*width, *height) =>
        {
            return Err(invalid("windowSize"));
        }
        CommandOperation::ScrollToOffset { offset } if !offset.is_finite() || *offset < 0.0 => {
            return Err(invalid("offset"));
        }
        CommandOperation::OpenSurface {
            title,
            width,
            height,
            options,
        } => {
            if title.chars().count() > 256 || !valid_surface_size(*width, *height) {
                return Err(invalid("openSurface"));
            }
            if options
                .as_ref()
                .is_some_and(|options| decode_window_options(wire_window_options(options)).is_err())
            {
                return Err(invalid("openSurface.options"));
            }
        }
        CommandOperation::FileDialogOpen {
            title,
            directories: _,
            multiple: _,
        } if title.chars().count() > 256 => return Err(invalid("fileDialogOpen")),
        CommandOperation::ResolveCloseRequest {
            request_id: _,
            allow: _,
        } => {}
        CommandOperation::WriteTextFile { path, content } => {
            if !valid_file_path(path) || content.len() > MAX_FILE_WRITE_BYTES {
                return Err(invalid("writeTextFile"));
            }
        }
        CommandOperation::ClipboardWriteImage { image } => {
            if ClipboardImageFormatCode::try_from(image.format).is_err()
                || image.bytes.is_empty()
                || image.bytes.len() > MAX_CLIPBOARD_IMAGE_BYTES
            {
                return Err(invalid("clipboardWriteImage"));
            }
        }
        CommandOperation::SetKeybindings { bindings } => {
            if bindings.len() > 64 || bindings.iter().any(|binding| !valid_keybinding(binding)) {
                return Err(invalid("keybindings"));
            }
        }
        CommandOperation::ShowNotification {
            title,
            body,
            actions,
        } => {
            if title.len() > 256
                || body.len() > 1024
                || actions.as_ref().is_some_and(|actions| {
                    actions.len() > 3
                        || actions
                            .iter()
                            .any(|action| !valid_notification_action(action))
                })
            {
                return Err(invalid("notification"));
            }
        }
        CommandOperation::SetTitle { title } if title.is_empty() || title.chars().count() > 256 => {
            return Err(invalid("title"));
        }
        CommandOperation::OpenUrl { url } if !valid_http_url(url) => return Err(invalid("url")),
        CommandOperation::ClipboardWrite { text } if text.len() > MAX_CLIPBOARD_TEXT_BYTES => {
            return Err(invalid("clipboardWrite"));
        }
        CommandOperation::SetClosePolicy { policy }
            if !matches!(policy.as_str(), "allow" | "require-confirmation") =>
        {
            return Err(invalid("closePolicy"));
        }
        CommandOperation::ReadTextFile { path } | CommandOperation::LoadFont { path }
            if !valid_file_path(path) =>
        {
            return Err(invalid("path"));
        }
        _ => {}
    }
    Ok(())
}

fn valid_file_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 1024
        && !path.chars().any(char::is_control)
        && std::path::Path::new(path).is_absolute()
}

fn valid_http_url(url: &str) -> bool {
    if url.is_empty() || url.len() > 2048 || url.chars().any(char::is_whitespace) {
        return false;
    }
    url.strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .is_some_and(|host| !host.is_empty())
}

fn valid_keybinding(value: &KeybindingDefinition) -> bool {
    !value.keystrokes.is_empty()
        && value.keystrokes.len() <= 64
        && !value.keystrokes.chars().any(char::is_control)
        && !value.action_name.is_empty()
        && value.action_name.chars().count() <= 64
        && !value.action_name.chars().any(char::is_control)
}

fn valid_menu_text(value: &str) -> bool {
    !value.is_empty() && value.chars().count() <= 256 && !value.chars().any(char::is_control)
}

fn valid_notification_action(value: &NotificationActionDefinition) -> bool {
    !value.id.is_empty()
        && value.id.len() <= 64
        && !value.label.is_empty()
        && value.label.len() <= 256
        && !value.id.chars().any(char::is_control)
        && !value.label.chars().any(char::is_control)
}

fn valid_surface_size(width: u32, height: u32) -> bool {
    (width == 0 && height == 0)
        || (width > 0
            && width <= MAX_WINDOW_DIMENSION
            && height > 0
            && height <= MAX_WINDOW_DIMENSION)
}

fn decode_menu_depth(
    value: generated::MenuDefinition<'_>,
    depth: usize,
) -> Result<MenuDefinition, ProtocolError> {
    if depth > 16 {
        return Err(ProtocolError::InvalidCommandPayload);
    }
    let title = value.title.ok_or(ProtocolError::InvalidCommandPayload)?;
    if !valid_menu_text(title) {
        return Err(ProtocolError::InvalidCommandPayload);
    }
    let items = value.items.ok_or(ProtocolError::InvalidCommandPayload)?;
    if items.len() > 1024 {
        return Err(ProtocolError::InvalidCommandPayload);
    }
    let items = items
        .into_iter()
        .map(|item| {
            let value = item.value.ok_or(ProtocolError::InvalidCommandPayload)?;
            match value {
                generated::MenuItemValue::MenuSeparator {} => Ok(MenuItemDefinition::Separator),
                generated::MenuItemValue::MenuAction {
                    name,
                    disabled,
                    checked,
                } => {
                    let name = name.ok_or(ProtocolError::InvalidCommandPayload)?;
                    if !valid_menu_text(name) {
                        return Err(ProtocolError::InvalidCommandPayload);
                    }
                    Ok(MenuItemDefinition::Action {
                        name: name.to_owned(),
                        disabled: disabled.ok_or(ProtocolError::InvalidCommandPayload)?,
                        checked: checked.ok_or(ProtocolError::InvalidCommandPayload)?,
                    })
                }
                generated::MenuItemValue::MenuSubmenu { menu } => {
                    Ok(MenuItemDefinition::Submenu(decode_menu_depth(
                        menu.ok_or(ProtocolError::InvalidCommandPayload)?,
                        depth + 1,
                    )?))
                }
                generated::MenuItemValue::Unknown => Err(ProtocolError::InvalidCommandPayload),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MenuDefinition {
        title: title.to_owned(),
        items,
    })
}

fn decode_menus(
    value: Vec<generated::MenuDefinition<'_>>,
) -> Result<Vec<MenuDefinition>, ProtocolError> {
    if value.len() > 64 {
        return Err(ProtocolError::InvalidCommandPayload);
    }
    value
        .into_iter()
        .map(|menu| decode_menu_depth(menu, 0))
        .collect()
}

fn decode_command_operation(
    kind: CommandKind,
    payload: Option<generated::CommandPayload<'_>>,
) -> Result<CommandOperation, ProtocolError> {
    let invalid = || ProtocolError::InvalidCommandPayload;
    match (kind, payload) {
        (
            CommandKind::OpenPopup,
            Some(generated::CommandPayload::OpenPopupCommand {
                anchor_node_id,
                width,
                height,
                placement,
                gap,
            }),
        ) => Ok(CommandOperation::OpenPopup {
            anchor_node_id: anchor_node_id.ok_or_else(invalid)?,
            width: width.ok_or_else(invalid)?,
            height: height.ok_or_else(invalid)?,
            placement: placement.ok_or_else(invalid)?,
            gap: gap.ok_or_else(invalid)?,
        }),
        (
            CommandKind::ClosePopup,
            Some(generated::CommandPayload::ClosePopupCommand { request_id }),
        ) => Ok(CommandOperation::ClosePopup {
            request_id: request_id.ok_or_else(invalid)?,
        }),
        (
            CommandKind::ConfigureApplication,
            Some(generated::CommandPayload::ConfigureApplicationCommand {
                keep_alive,
                quit,
                acknowledged_sequence,
            }),
        ) => Ok(CommandOperation::ConfigureApplication {
            keep_alive: keep_alive.ok_or_else(structural_error)?,
            quit: quit.ok_or_else(structural_error)?,
            acknowledged_sequence: acknowledged_sequence.ok_or_else(structural_error)?,
        }),
        (
            CommandKind::CancelNative,
            Some(generated::CommandPayload::CancelNativeCommand { request_id }),
        ) => {
            let request_id = request_id.filter(|id| *id != 0).ok_or_else(invalid)?;
            Ok(CommandOperation::CancelNative { request_id })
        }
        (
            CommandKind::InvokeNative,
            Some(generated::CommandPayload::InvokeNativeCommand {
                module_id,
                module_digest,
                function_id,
                args,
            }),
        ) => {
            let module_id = module_id.ok_or_else(invalid)?;
            let module_digest = module_digest.ok_or_else(invalid)?;
            let function_id = function_id.ok_or_else(invalid)?;
            let args = args.ok_or_else(invalid)?;
            if module_id.len() != 16
                || module_digest.len() != 32
                || function_id == 0
                || args.len() > MAX_NATIVE_CALL_BYTES
            {
                return Err(invalid());
            }
            Ok(CommandOperation::InvokeNative {
                module_id: module_id.as_ref().try_into().map_err(|_| invalid())?,
                module_digest: module_digest.as_ref().try_into().map_err(|_| invalid())?,
                function_id,
                args: args.as_ref().to_vec(),
            })
        }
        (
            CommandKind::SetSelection,
            Some(generated::CommandPayload::U32PairCommand { first, second }),
        ) => {
            let start = first.ok_or_else(invalid)?;
            let end = second.ok_or_else(invalid)?;
            if start > end {
                return Err(invalid());
            }
            Ok(CommandOperation::SetSelection { start, end })
        }
        (
            CommandKind::ScrollToIndex,
            Some(generated::CommandPayload::U32PairCommand { first, second }),
        ) => Ok(CommandOperation::ScrollToIndex {
            index: first.ok_or_else(invalid)?,
            alignment: second.ok_or_else(invalid)?,
        }),
        (
            CommandKind::ResizeWindow,
            Some(generated::CommandPayload::U32PairCommand { first, second }),
        ) => {
            let width = first.ok_or_else(invalid)?;
            let height = second.ok_or_else(invalid)?;
            if !valid_surface_size(width, height) {
                return Err(invalid());
            }
            Ok(CommandOperation::ResizeWindow { width, height })
        }
        (
            CommandKind::ResolveCloseRequest,
            Some(generated::CommandPayload::CloseResolutionCommand { request_id, allow }),
        ) => Ok(CommandOperation::ResolveCloseRequest {
            request_id: request_id.ok_or_else(invalid)?,
            allow: allow.ok_or_else(invalid)?,
        }),
        (CommandKind::ScrollToOffset, Some(generated::CommandPayload::FloatCommand { value })) => {
            let offset = value.ok_or_else(invalid)?;
            if !offset.is_finite() || offset < 0.0 {
                return Err(invalid());
            }
            Ok(CommandOperation::ScrollToOffset { offset })
        }
        (
            CommandKind::OpenSurface,
            Some(generated::CommandPayload::OpenSurfaceCommand {
                title,
                width,
                height,
                options,
            }),
        ) => {
            let title = title.ok_or_else(invalid)?;
            let width = width.ok_or_else(invalid)?;
            let height = height.ok_or_else(invalid)?;
            if title.chars().count() > 256 || !valid_surface_size(width, height) {
                return Err(invalid());
            }
            Ok(CommandOperation::OpenSurface {
                title: title.to_owned(),
                width,
                height,
                options: options.map(decode_window_options).transpose()?,
            })
        }
        (
            CommandKind::FileDialogOpen,
            Some(generated::CommandPayload::FileDialogOpenCommand {
                title,
                directories,
                multiple,
            }),
        ) => {
            let title = title.ok_or_else(invalid)?;
            if title.chars().count() > 256 {
                return Err(invalid());
            }
            Ok(CommandOperation::FileDialogOpen {
                title: title.to_owned(),
                directories: directories.ok_or_else(invalid)?,
                multiple: multiple.ok_or_else(invalid)?,
            })
        }
        (
            CommandKind::ShowNotification,
            Some(generated::CommandPayload::NotificationCommand {
                title,
                body,
                actions,
            }),
        ) => {
            let title = title.ok_or_else(invalid)?;
            let body = body.ok_or_else(invalid)?;
            let actions = actions.map(decode_notification_actions).transpose()?;
            if title.len() > 256
                || body.len() > 1024
                || actions.as_ref().is_some_and(|actions| {
                    actions.len() > 3
                        || actions
                            .iter()
                            .any(|action| !valid_notification_action(action))
                })
            {
                return Err(invalid());
            }
            Ok(CommandOperation::ShowNotification {
                title: title.to_owned(),
                body: body.to_owned(),
                actions,
            })
        }
        (CommandKind::SetMenus, Some(generated::CommandPayload::MenusCommand { menus })) => {
            Ok(CommandOperation::SetMenus {
                menus: decode_menus(menus.ok_or_else(invalid)?)?,
            })
        }
        (
            CommandKind::SetKeybindings,
            Some(generated::CommandPayload::KeybindingsCommand { bindings }),
        ) => {
            let bindings = decode_keybindings(bindings.ok_or_else(invalid)?)?;
            if bindings.len() > 64 || bindings.iter().any(|binding| !valid_keybinding(binding)) {
                return Err(invalid());
            }
            Ok(CommandOperation::SetKeybindings { bindings })
        }
        (
            CommandKind::ClipboardWriteImage,
            Some(generated::CommandPayload::ClipboardImageCommand { format, bytes }),
        ) => Ok(CommandOperation::ClipboardWriteImage {
            image: decode_image(format, bytes)?,
        }),
        (
            CommandKind::WriteTextFile,
            Some(generated::CommandPayload::StringPairCommand { path, content }),
        ) => {
            let path = path.ok_or_else(invalid)?;
            let content = content.ok_or_else(invalid)?;
            if !valid_file_path(path) || content.len() > MAX_FILE_WRITE_BYTES {
                return Err(invalid());
            }
            Ok(CommandOperation::WriteTextFile {
                path: path.to_owned(),
                content: content.to_owned(),
            })
        }
        (
            CommandKind::SetTitle
            | CommandKind::OpenUrl
            | CommandKind::ClipboardWrite
            | CommandKind::FileDialogSave
            | CommandKind::SetClosePolicy
            | CommandKind::ReadTextFile
            | CommandKind::LoadFont,
            Some(generated::CommandPayload::TextCommand { value }),
        ) => {
            let value = value.ok_or_else(invalid)?;
            match kind {
                CommandKind::SetTitle if value.is_empty() => return Err(invalid()),
                CommandKind::OpenUrl if !valid_http_url(value) => return Err(invalid()),
                CommandKind::ClipboardWrite if value.len() > MAX_CLIPBOARD_TEXT_BYTES => {
                    return Err(invalid());
                }
                CommandKind::ReadTextFile | CommandKind::LoadFont if !valid_file_path(value) => {
                    return Err(invalid());
                }
                CommandKind::SetClosePolicy
                    if !matches!(value, "allow" | "require-confirmation") =>
                {
                    return Err(invalid());
                }
                _ if value.chars().count() > 256 => return Err(invalid()),
                _ => {}
            }
            Ok(match kind {
                CommandKind::SetTitle => CommandOperation::SetTitle {
                    title: value.to_owned(),
                },
                CommandKind::OpenUrl => CommandOperation::OpenUrl {
                    url: value.to_owned(),
                },
                CommandKind::ClipboardWrite => CommandOperation::ClipboardWrite {
                    text: value.to_owned(),
                },
                CommandKind::FileDialogSave => CommandOperation::FileDialogSave {
                    default_name: value.to_owned(),
                },
                CommandKind::SetClosePolicy => CommandOperation::SetClosePolicy {
                    policy: value.to_owned(),
                },
                CommandKind::ReadTextFile => CommandOperation::ReadTextFile {
                    path: value.to_owned(),
                },
                CommandKind::LoadFont => CommandOperation::LoadFont {
                    path: value.to_owned(),
                },
                _ => return Err(invalid()),
            })
        }
        (CommandKind::Focus, None) => Ok(CommandOperation::Focus),
        (CommandKind::Blur, None) => Ok(CommandOperation::Blur),
        (CommandKind::ScrollToEnd, None) => Ok(CommandOperation::ScrollToEnd),
        (CommandKind::ZoomWindow, None) => Ok(CommandOperation::ZoomWindow),
        (CommandKind::ToggleFullscreen, None) => Ok(CommandOperation::ToggleFullscreen),
        (CommandKind::FocusNext, None) => Ok(CommandOperation::FocusNext),
        (CommandKind::FocusPrev, None) => Ok(CommandOperation::FocusPrev),
        (CommandKind::GetWindowSize, None) => Ok(CommandOperation::GetWindowSize),
        (CommandKind::GetFocus, None) => Ok(CommandOperation::GetFocus),
        (CommandKind::ClipboardRead, None) => Ok(CommandOperation::ClipboardRead),
        (CommandKind::ClipboardReadImage, None) => Ok(CommandOperation::ClipboardReadImage),
        (CommandKind::MinimizeWindow, None) => Ok(CommandOperation::MinimizeWindow),
        (CommandKind::GetWindowBounds, None) => Ok(CommandOperation::GetWindowBounds),
        (CommandKind::GetWindowState, None) => Ok(CommandOperation::GetWindowState),
        (CommandKind::ActivateWindow, None) => Ok(CommandOperation::ActivateWindow),
        (CommandKind::GetScrollOffset, None) => Ok(CommandOperation::GetScrollOffset),
        (_, None) | (_, Some(_)) => Err(invalid()),
    }
}

fn decode_command_body_value(body: generated::Body<'_>) -> Result<Command, ProtocolError> {
    let generated::Body::Command {
        surface_id,
        epoch,
        after_revision,
        request_id,
        node_id,
        kind,
        payload,
    } = body
    else {
        return Err(ProtocolError::WrongMessageType(COMMAND_MESSAGE));
    };
    let kind = CommandKind::try_from(command_kind_value(kind.ok_or_else(structural_error)?))
        .map_err(|_| ProtocolError::UnknownCommand(0))?;
    let meta = CommandMeta {
        surface_id: surface_id.ok_or_else(structural_error)?,
        epoch: epoch.ok_or_else(structural_error)?,
        after_revision: after_revision.ok_or_else(structural_error)?,
        request_id: request_id.ok_or_else(structural_error)?,
        node_id: node_id.ok_or_else(structural_error)?,
    };
    let command = Command::new(meta, decode_command_operation(kind, payload)?);
    validate_command(&command)?;
    Ok(command)
}

pub(super) fn decode_command(payload: &[u8]) -> Result<Command, ProtocolError> {
    decode_command_body_value(
        decode_envelope(payload)?
            .body
            .ok_or_else(structural_error)?,
    )
}
fn wire_text_input<'a>(value: &'a TextInputEvent) -> generated::EventPayload<'a> {
    generated::EventPayload::TextInputEventData {
        text: Some(&value.text),
        selection_start: Some(value.selection_start),
        selection_end: Some(value.selection_end),
        marked_start: value.marked_start,
        marked_end: value.marked_end,
        edit_seq: Some(value.edit_seq),
        reversed: Some(value.reversed),
    }
}

fn wire_event_payload<'a>(value: &'a EventPayload) -> Option<generated::EventPayload<'a>> {
    match value {
        EventPayload::ApplicationActivation {
            target_surface_id,
            reason,
            urls,
        } => Some(generated::EventPayload::ApplicationActivationEvent {
            target_surface_id: Some(*target_surface_id),
            reason: Some(reason),
            urls: Some(urls.iter().map(String::as_str).collect()),
        }),
        EventPayload::Press
        | EventPayload::Focus
        | EventPayload::Blur
        | EventPayload::Hover
        | EventPayload::SurfaceClosed => None,
        EventPayload::TextInputChange(value)
        | EventPayload::TextInputSelection(value)
        | EventPayload::FocusTextInput(value)
        | EventPayload::BlurTextInput(value) => Some(wire_text_input(value)),
        EventPayload::CommandResult(value) => Some(generated::EventPayload::CommandResult {
            request_id: Some(value.request_id),
            command: Some(value.command.into()),
            node_id: Some(value.node_id),
            success: Some(value.success),
            error: value.error.as_deref(),
            value: value.value.as_ref().map(wire_command_value),
        }),
        EventPayload::VisibleRange { start, end } => {
            Some(generated::EventPayload::VisibleRangeEvent {
                start: Some(*start),
                end: Some(*end),
            })
        }
        EventPayload::AnimationComplete { generation } => {
            Some(generated::EventPayload::AnimationCompleteEvent {
                generation: Some(*generation),
            })
        }
        EventPayload::Key(value) => Some(generated::EventPayload::KeyEvent {
            key: Some(&value.key),
            modifiers: Some(value.modifiers.iter().map(String::as_str).collect()),
            action: Some(match value.action {
                KeyAction::Down => EVENT_KEY_DOWN,
                KeyAction::Repeat => EVENT_KEY_REPEAT,
                KeyAction::Up => EVENT_KEY_UP,
            }),
        }),
        EventPayload::Pointer(value) => Some(generated::EventPayload::PointerEvent {
            button: Some(value.button),
            modifiers: Some(value.modifiers.iter().map(String::as_str).collect()),
            action: Some(value.action),
            click_count: Some(value.click_count),
            x: Some(value.x),
            y: Some(value.y),
        }),
        EventPayload::PointerMove(value) => Some(generated::EventPayload::PointerMoveEvent {
            modifiers: Some(value.modifiers.iter().map(String::as_str).collect()),
            x: Some(value.x),
            y: Some(value.y),
        }),
        EventPayload::Scroll(value) => Some(generated::EventPayload::ScrollEvent {
            delta_kind: Some(value.delta_kind),
            dx: Some(value.dx),
            dy: Some(value.dy),
            x: Some(value.x),
            y: Some(value.y),
            modifiers: Some(value.modifiers.iter().map(String::as_str).collect()),
        }),
        EventPayload::Submit { text } => {
            Some(generated::EventPayload::SubmitEvent { text: Some(text) })
        }
        EventPayload::WindowResize {
            width,
            height,
            scale_factor,
        } => Some(generated::EventPayload::WindowResizeEvent {
            width: Some(*width),
            height: Some(*height),
            scale_factor: Some(*scale_factor),
        }),
        EventPayload::WindowActivation { active } => {
            Some(generated::EventPayload::WindowActivationEvent {
                active: Some(*active),
            })
        }
        EventPayload::EventAction { action } => Some(generated::EventPayload::ActionEvent {
            action: Some(action),
        }),
        EventPayload::WindowAppearance { appearance } => {
            Some(generated::EventPayload::WindowAppearanceEvent {
                appearance: Some(match appearance {
                    WindowAppearance::Light => generated::WindowAppearance::Light,
                    WindowAppearance::Dark => generated::WindowAppearance::Dark,
                }),
            })
        }
        EventPayload::Layout {
            x,
            y,
            width,
            height,
        } => Some(generated::EventPayload::LayoutEvent {
            x: Some(*x),
            y: Some(*y),
            width: Some(*width),
            height: Some(*height),
        }),
        EventPayload::DragOver { drag_type } => Some(generated::EventPayload::DragOverEvent {
            drag_type: Some(drag_type),
        }),
        EventPayload::DragDrop { drag_type } => Some(generated::EventPayload::DragDropEvent {
            drag_type: Some(drag_type),
        }),
        EventPayload::ExternalFileDrop { paths } => {
            Some(generated::EventPayload::ExternalFileDropEvent {
                paths: Some(paths.iter().map(String::as_str).collect()),
            })
        }
        EventPayload::NotificationResponse(value) => {
            Some(generated::EventPayload::NotificationResponseEvent {
                tag: Some(&value.tag),
                action_id: value.action_id.as_deref(),
            })
        }
        EventPayload::PointerDownOutside { x, y } => {
            Some(generated::EventPayload::PointerDownOutsideEvent {
                x: Some(*x),
                y: Some(*y),
            })
        }
        EventPayload::CloseRequested { request_id } => {
            Some(generated::EventPayload::CloseRequestedEvent {
                request_id: Some(*request_id),
            })
        }
        EventPayload::Extension { event_id, fields } => {
            Some(generated::EventPayload::ExtensionEvent {
                event_id: Some(*event_id),
                fields: Some(wire_extension_fields(fields)),
            })
        }
    }
}

fn decode_text_input(
    text: Option<&str>,
    selection_start: Option<u32>,
    selection_end: Option<u32>,
    marked_start: Option<u32>,
    marked_end: Option<u32>,
    edit_seq: Option<u32>,
    reversed: Option<bool>,
) -> Result<TextInputEvent, ProtocolError> {
    if marked_start.is_some() != marked_end.is_some()
        || marked_start
            .zip(marked_end)
            .is_some_and(|(start, end)| start > end)
    {
        return Err(ProtocolError::InvalidTextInputEvent);
    }
    let selection_start = selection_start.ok_or(ProtocolError::InvalidTextInputEvent)?;
    let selection_end = selection_end.ok_or(ProtocolError::InvalidTextInputEvent)?;
    if selection_start > selection_end {
        return Err(ProtocolError::InvalidTextInputEvent);
    }
    Ok(TextInputEvent {
        text: text.ok_or(ProtocolError::InvalidTextInputEvent)?.to_owned(),
        selection_start,
        selection_end,
        marked_start,
        marked_end,
        edit_seq: edit_seq.ok_or(ProtocolError::InvalidTextInputEvent)?,
        reversed: reversed.ok_or(ProtocolError::InvalidTextInputEvent)?,
    })
}
fn decode_event_payload(
    event_kind: EventKind,
    value: generated::EventPayload<'_>,
) -> Result<EventPayload, ProtocolError> {
    let invalid = || ProtocolError::InvalidEventPayload;
    match (event_kind, value) {
        (
            EventKind::ApplicationActivation,
            generated::EventPayload::ApplicationActivationEvent {
                target_surface_id,
                reason,
                urls,
            },
        ) => Ok(EventPayload::ApplicationActivation {
            target_surface_id: target_surface_id.ok_or_else(structural_error)?,
            reason: reason.ok_or_else(structural_error)?.to_owned(),
            urls: urls
                .ok_or_else(structural_error)?
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }),

        (
            EventKind::Change,
            generated::EventPayload::TextInputEventData {
                text,
                selection_start,
                selection_end,
                marked_start,
                marked_end,
                edit_seq,
                reversed,
            },
        ) => Ok(EventPayload::TextInputChange(decode_text_input(
            text,
            selection_start,
            selection_end,
            marked_start,
            marked_end,
            edit_seq,
            reversed,
        )?)),
        (
            EventKind::Selection,
            generated::EventPayload::TextInputEventData {
                text,
                selection_start,
                selection_end,
                marked_start,
                marked_end,
                edit_seq,
                reversed,
            },
        ) => Ok(EventPayload::TextInputSelection(decode_text_input(
            text,
            selection_start,
            selection_end,
            marked_start,
            marked_end,
            edit_seq,
            reversed,
        )?)),
        (
            EventKind::Focus,
            generated::EventPayload::TextInputEventData {
                text,
                selection_start,
                selection_end,
                marked_start,
                marked_end,
                edit_seq,
                reversed,
            },
        ) => Ok(EventPayload::FocusTextInput(decode_text_input(
            text,
            selection_start,
            selection_end,
            marked_start,
            marked_end,
            edit_seq,
            reversed,
        )?)),
        (
            EventKind::Blur,
            generated::EventPayload::TextInputEventData {
                text,
                selection_start,
                selection_end,
                marked_start,
                marked_end,
                edit_seq,
                reversed,
            },
        ) => Ok(EventPayload::BlurTextInput(decode_text_input(
            text,
            selection_start,
            selection_end,
            marked_start,
            marked_end,
            edit_seq,
            reversed,
        )?)),
        (
            EventKind::CommandResult,
            generated::EventPayload::CommandResult {
                request_id,
                command,
                node_id,
                success,
                error,
                value,
            },
        ) => Ok(EventPayload::CommandResult(CommandResult {
            request_id: request_id.ok_or_else(invalid)?,
            command: CommandKind::try_from(command.ok_or_else(invalid)?).map_err(|_| invalid())?,
            node_id: node_id.ok_or_else(invalid)?,
            success: success.ok_or_else(invalid)?,
            error: error.map(str::to_owned),
            value: value.map(decode_command_value).transpose()?,
        })),
        (EventKind::VisibleRange, generated::EventPayload::VisibleRangeEvent { start, end }) => {
            Ok(EventPayload::VisibleRange {
                start: start.ok_or_else(invalid)?,
                end: end.ok_or_else(invalid)?,
            })
        }
        (
            EventKind::AnimationComplete,
            generated::EventPayload::AnimationCompleteEvent { generation },
        ) => Ok(EventPayload::AnimationComplete {
            generation: generation.ok_or_else(invalid)?,
        }),
        (
            EventKind::Key,
            generated::EventPayload::KeyEvent {
                key,
                modifiers,
                action,
            },
        ) => Ok(EventPayload::Key(KeyEvent {
            key: key.ok_or_else(invalid)?.to_owned(),
            modifiers: modifiers
                .ok_or_else(invalid)?
                .into_iter()
                .map(str::to_owned)
                .collect(),
            action: match action.ok_or_else(invalid)? {
                EVENT_KEY_DOWN => KeyAction::Down,
                EVENT_KEY_REPEAT => KeyAction::Repeat,
                EVENT_KEY_UP => KeyAction::Up,
                _ => return Err(invalid()),
            },
        })),
        (
            EventKind::Pointer,
            generated::EventPayload::PointerEvent {
                button,
                modifiers,
                action,
                click_count,
                x,
                y,
            },
        ) => Ok(EventPayload::Pointer(PointerEvent {
            button: button.ok_or_else(invalid)?,
            modifiers: modifiers
                .ok_or_else(invalid)?
                .into_iter()
                .map(str::to_owned)
                .collect(),
            action: action.ok_or_else(invalid)?,
            click_count: click_count.ok_or_else(invalid)?,
            x: x.ok_or_else(invalid)?,
            y: y.ok_or_else(invalid)?,
        })),
        (EventKind::Pointer, generated::EventPayload::PointerMoveEvent { modifiers, x, y }) => {
            Ok(EventPayload::PointerMove(PointerMoveEvent {
                modifiers: modifiers
                    .ok_or_else(invalid)?
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
                x: x.ok_or_else(invalid)?,
                y: y.ok_or_else(invalid)?,
            }))
        }
        (
            EventKind::Scroll,
            generated::EventPayload::ScrollEvent {
                delta_kind,
                dx,
                dy,
                x,
                y,
                modifiers,
            },
        ) => Ok(EventPayload::Scroll(ScrollEvent {
            delta_kind: delta_kind.ok_or_else(invalid)?,
            dx: dx.ok_or_else(invalid)?,
            dy: dy.ok_or_else(invalid)?,
            x: x.ok_or_else(invalid)?,
            y: y.ok_or_else(invalid)?,
            modifiers: modifiers
                .ok_or_else(invalid)?
                .into_iter()
                .map(str::to_owned)
                .collect(),
        })),
        (EventKind::Submit, generated::EventPayload::SubmitEvent { text }) => {
            Ok(EventPayload::Submit {
                text: text.ok_or_else(invalid)?.to_owned(),
            })
        }
        (
            EventKind::WindowResize,
            generated::EventPayload::WindowResizeEvent {
                width,
                height,
                scale_factor,
            },
        ) => Ok(EventPayload::WindowResize {
            width: width.ok_or_else(invalid)?,
            height: height.ok_or_else(invalid)?,
            scale_factor: scale_factor.ok_or_else(invalid)?,
        }),
        (
            EventKind::WindowActivation,
            generated::EventPayload::WindowActivationEvent { active },
        ) => Ok(EventPayload::WindowActivation {
            active: active.ok_or_else(invalid)?,
        }),
        (EventKind::Extension, generated::EventPayload::ExtensionEvent { event_id, fields }) => {
            Ok(EventPayload::Extension {
                event_id: event_id.ok_or_else(invalid)?,
                fields: decode_extension_fields(fields).map_err(|_| invalid())?,
            })
        }
        (EventKind::Action, generated::EventPayload::ActionEvent { action }) => {
            Ok(EventPayload::EventAction {
                action: action.ok_or_else(invalid)?.to_owned(),
            })
        }
        (
            EventKind::WindowAppearance,
            generated::EventPayload::WindowAppearanceEvent { appearance },
        ) => Ok(EventPayload::WindowAppearance {
            appearance: match appearance.ok_or_else(invalid)? {
                generated::WindowAppearance::Light => WindowAppearance::Light,
                generated::WindowAppearance::Dark => WindowAppearance::Dark,
                generated::WindowAppearance::Unspecified => return Err(invalid()),
            },
        }),
        (
            EventKind::Layout,
            generated::EventPayload::LayoutEvent {
                x,
                y,
                width,
                height,
            },
        ) => Ok(EventPayload::Layout {
            x: x.ok_or_else(invalid)?,
            y: y.ok_or_else(invalid)?,
            width: width.ok_or_else(invalid)?,
            height: height.ok_or_else(invalid)?,
        }),
        (EventKind::Drag, generated::EventPayload::DragOverEvent { drag_type }) => {
            Ok(EventPayload::DragOver {
                drag_type: drag_type.ok_or_else(invalid)?.to_owned(),
            })
        }
        (EventKind::Drag, generated::EventPayload::DragDropEvent { drag_type }) => {
            Ok(EventPayload::DragDrop {
                drag_type: drag_type.ok_or_else(invalid)?.to_owned(),
            })
        }
        (EventKind::Drag, generated::EventPayload::ExternalFileDropEvent { paths }) => {
            Ok(EventPayload::ExternalFileDrop {
                paths: paths
                    .ok_or_else(invalid)?
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            })
        }
        (
            EventKind::NotificationResponse,
            generated::EventPayload::NotificationResponseEvent { tag, action_id },
        ) => Ok(EventPayload::NotificationResponse(
            NotificationResponseEvent {
                tag: tag.ok_or_else(invalid)?.to_owned(),
                action_id: action_id.map(str::to_owned),
            },
        )),
        (
            EventKind::PointerDownOutside,
            generated::EventPayload::PointerDownOutsideEvent { x, y },
        ) => Ok(EventPayload::PointerDownOutside {
            x: x.ok_or_else(invalid)?,
            y: y.ok_or_else(invalid)?,
        }),
        (
            EventKind::CloseRequested,
            generated::EventPayload::CloseRequestedEvent { request_id },
        ) => Ok(EventPayload::CloseRequested {
            request_id: request_id.ok_or_else(invalid)?,
        }),
        _ => Err(invalid()),
    }
}

fn valid_modifiers(modifiers: &[String]) -> bool {
    let names = ["cmd", "ctrl", "alt", "shift", "function"];
    modifiers.len() <= names.len()
        && modifiers
            .iter()
            .all(|modifier| names.contains(&modifier.as_str()))
        && modifiers
            .iter()
            .enumerate()
            .all(|(index, modifier)| !modifiers[..index].contains(modifier))
}

fn valid_command_value(value: &CommandValue) -> bool {
    match value {
        CommandValue::Number(_) => true,
        CommandValue::Pair((width, height)) => {
            width.is_finite() && *width >= 0.0 && height.is_finite() && *height >= 0.0
        }
        CommandValue::Bool(_) => true,
        CommandValue::Text(value) => value.len() <= MAX_CLIPBOARD_TEXT_BYTES,
        CommandValue::Paths(values) => {
            !values.is_empty()
                && values
                    .iter()
                    .all(|value| !value.is_empty() && value.len() <= MAX_FILE_READ_BYTES)
        }
        CommandValue::FileText(value) => value.len() <= MAX_FILE_READ_BYTES,
        CommandValue::Image(value) => {
            ClipboardImageFormatCode::try_from(value.format).is_ok()
                && !value.bytes.is_empty()
                && value.bytes.len() <= MAX_CLIPBOARD_IMAGE_BYTES
        }
        CommandValue::Bounds((x, y, width, height)) => {
            x.is_finite()
                && y.is_finite()
                && width.is_finite()
                && height.is_finite()
                && *width >= 0.0
                && *height >= 0.0
        }
        CommandValue::WindowState(_) => true,
        CommandValue::ScrollOffset(value) => value.is_finite() && *value >= 0.0,
        CommandValue::Bytes(value) => value.len() <= MAX_NATIVE_CALL_BYTES,
    }
}

fn valid_event_payload(payload: &EventPayload) -> bool {
    match payload {
        EventPayload::ApplicationActivation {
            target_surface_id,
            reason,
            urls,
        } => {
            *target_surface_id > 0
                && matches!(reason.as_str(), "launch" | "reopen" | "open-urls")
                && urls.len() <= 64
                && (if reason == "open-urls" {
                    !urls.is_empty()
                } else {
                    urls.is_empty()
                })
                && urls.iter().all(|url| {
                    !url.is_empty() && url.len() <= 4096 && !url.chars().any(char::is_control)
                })
        }
        EventPayload::Press
        | EventPayload::Focus
        | EventPayload::Blur
        | EventPayload::Hover
        | EventPayload::SurfaceClosed => true,
        EventPayload::TextInputChange(value)
        | EventPayload::TextInputSelection(value)
        | EventPayload::FocusTextInput(value)
        | EventPayload::BlurTextInput(value) => {
            value.text.len() <= MAX_CLIPBOARD_TEXT_BYTES
                && value.selection_start <= value.selection_end
                && value.marked_start.is_some() == value.marked_end.is_some()
                && value
                    .marked_start
                    .zip(value.marked_end)
                    .is_none_or(|(start, end)| start <= end)
        }
        EventPayload::CommandResult(value) => {
            value
                .error
                .as_ref()
                .is_none_or(|error| error.len() <= MAX_CLIPBOARD_TEXT_BYTES)
                && value.value.as_ref().is_none_or(valid_command_value)
        }
        EventPayload::VisibleRange { start, end } => start <= end,
        EventPayload::AnimationComplete { .. } => true,
        EventPayload::Key(KeyEvent {
            key,
            modifiers,
            action,
        }) => {
            !key.is_empty()
                && key.len() <= MAX_CLIPBOARD_TEXT_BYTES
                && valid_modifiers(modifiers)
                && matches!(action, KeyAction::Down | KeyAction::Repeat | KeyAction::Up)
        }
        EventPayload::Pointer(PointerEvent {
            button,
            modifiers,
            action,
            click_count,
            x,
            y,
        }) => {
            (1..=5).contains(button)
                && valid_modifiers(modifiers)
                && (*action == EVENT_POINTER_DOWN || *action == EVENT_POINTER_UP)
                && *click_count > 0
                && x.is_finite()
                && *x >= 0.0
                && y.is_finite()
                && *y >= 0.0
        }
        EventPayload::PointerMove(PointerMoveEvent { x, y, modifiers }) => {
            x.is_finite() && *x >= 0.0 && y.is_finite() && *y >= 0.0 && valid_modifiers(modifiers)
        }
        EventPayload::Scroll(ScrollEvent {
            delta_kind,
            dx,
            dy,
            x,
            y,
            modifiers,
        }) => {
            (*delta_kind == SCROLL_DELTA_PIXELS || *delta_kind == SCROLL_DELTA_LINES)
                && [*dx, *dy, *x, *y].into_iter().all(f32::is_finite)
                && valid_modifiers(modifiers)
        }
        EventPayload::Submit { .. } => true,
        EventPayload::WindowResize {
            width,
            height,
            scale_factor,
        } => {
            width.is_finite()
                && *width >= 0.0
                && height.is_finite()
                && *height >= 0.0
                && scale_factor.is_finite()
                && *scale_factor > 0.0
        }
        EventPayload::WindowActivation { .. } => true,
        EventPayload::EventAction { action } => !action.is_empty() && action.len() <= 256,
        EventPayload::WindowAppearance { .. } => true,
        EventPayload::Layout {
            x,
            y,
            width,
            height,
        } => [*x, *y, *width, *height].into_iter().all(f32::is_finite),
        EventPayload::DragOver { drag_type } | EventPayload::DragDrop { drag_type } => {
            !drag_type.is_empty()
                && drag_type.len() <= 512
                && !drag_type.chars().any(char::is_control)
        }
        EventPayload::ExternalFileDrop { paths } => {
            !paths.is_empty()
                && paths.iter().all(|path| {
                    !path.is_empty() && path.len() <= 4096 && !path.chars().any(char::is_control)
                })
        }
        EventPayload::NotificationResponse(value) => {
            !value.tag.is_empty()
                && value.tag.len() <= 1024
                && value
                    .action_id
                    .as_ref()
                    .is_none_or(|action| !action.is_empty() && action.len() <= 64)
        }
        EventPayload::PointerDownOutside { x, y } => x.is_finite() && y.is_finite(),
        EventPayload::CloseRequested { .. } => true,
        EventPayload::Extension {
            event_id: _,
            fields,
        } => validate_extension_fields(fields).is_ok(),
    }
}

fn validate_event(event: &Event) -> Result<(), ProtocolError> {
    if matches!(event.payload, EventPayload::ApplicationActivation { .. })
        && (event.meta.surface_id != 0
            || event.meta.node_id != 0
            || event.meta.listener_id != 0
            || event.meta.revision != 0
            || event.meta.epoch == 0
            || event.meta.sequence == 0)
    {
        return Err(ProtocolError::InvalidEventPayload);
    }

    if !valid_event_payload(&event.payload) {
        return Err(ProtocolError::InvalidEventPayload);
    }
    if let EventPayload::CommandResult(result) = &event.payload {
        if result.command == CommandKind::InvokeNative {
            if event.meta.node_id == 0
                || result.node_id != event.meta.node_id
                || event.meta.listener_id != 0
                || (result.success
                    && (result.error.is_some()
                        || !matches!(result.value, Some(CommandValue::Bytes(_)))))
                || (!result.success && result.value.is_some())
            {
                return Err(ProtocolError::InvalidEventPayload);
            }
        } else if matches!(result.value, Some(CommandValue::Bytes(_))) {
            return Err(ProtocolError::InvalidEventPayload);
        }
    }
    if matches!(event.payload, EventPayload::SurfaceClosed)
        && (event.meta.node_id != 0 || event.meta.listener_id != 0)
    {
        return Err(ProtocolError::InvalidEventPayload);
    }
    if matches!(event.payload, EventPayload::Focus | EventPayload::Blur)
        && (event.meta.node_id == 0 || event.meta.listener_id == 0)
    {
        return Err(ProtocolError::InvalidEventPayload);
    }
    if matches!(
        event.payload,
        EventPayload::EventAction { .. }
            | EventPayload::WindowAppearance { .. }
            | EventPayload::NotificationResponse(_)
    ) && (event.meta.node_id != 1 || event.meta.listener_id != 0)
    {
        return Err(ProtocolError::InvalidEventPayload);
    }
    if matches!(
        event.payload,
        EventPayload::DragOver { .. }
            | EventPayload::DragDrop { .. }
            | EventPayload::ExternalFileDrop { .. }
            | EventPayload::PointerDownOutside { .. }
    ) && (event.meta.node_id == 0 || event.meta.listener_id == 0)
    {
        return Err(ProtocolError::InvalidEventPayload);
    }
    Ok(())
}

fn decode_event_body_value(body: generated::Body<'_>) -> Result<Event, ProtocolError> {
    let generated::Body::Event {
        surface_id,
        epoch,
        revision,
        sequence,
        node_id,
        listener_id,
        event_type,
        payload,
    } = body
    else {
        return Err(ProtocolError::WrongMessageType(EVENT_MESSAGE));
    };
    let event_kind =
        EventKind::try_from(event_kind_value(event_type.ok_or_else(structural_error)?))
            .map_err(|_| ProtocolError::UnknownEvent(0))?;
    let payload = match payload {
        Some(payload) => decode_event_payload(event_kind, payload)
            .map_err(|error| error.at(format!("body.event.payload(type={event_kind:?})")))?,
        None => match event_kind {
            EventKind::Press => EventPayload::Press,
            EventKind::Focus => EventPayload::Focus,
            EventKind::Blur => EventPayload::Blur,
            EventKind::Hover => EventPayload::Hover,
            EventKind::SurfaceClosed => EventPayload::SurfaceClosed,
            _ => return Err(ProtocolError::InvalidEventPayload),
        },
    };
    let event = Event::new(
        EventMeta {
            surface_id: surface_id.ok_or_else(structural_error)?,
            epoch: epoch.ok_or_else(structural_error)?,
            revision: revision.ok_or_else(structural_error)?,
            sequence: sequence.ok_or_else(structural_error)?,
            node_id: node_id.ok_or_else(structural_error)?,
            listener_id: listener_id.ok_or_else(structural_error)?,
        },
        payload,
    );
    validate_event(&event).map_err(|error| error.at(format!("body.event(type={event_kind:?})")))?;
    Ok(event)
}

pub(super) fn decode_event(payload: &[u8]) -> Result<Event, ProtocolError> {
    decode_event_body_value(
        decode_envelope(payload)?
            .body
            .ok_or_else(structural_error)?,
    )
}

pub(super) fn decode_message(
    payload: &[u8],
) -> Result<super::super::DecodedMessage, ProtocolError> {
    let body = decode_envelope(payload)?
        .body
        .ok_or_else(structural_error)?;
    match body {
        generated::Body::Snapshot { .. } => Ok(super::super::DecodedMessage::Snapshot(
            decode_snapshot_body(body)?,
        )),
        generated::Body::Patch { .. } => Ok(super::super::DecodedMessage::Patch(
            decode_patch_body(body)?,
        )),
        generated::Body::Command { .. } => Ok(super::super::DecodedMessage::Command(
            decode_command_body_value(body)?,
        )),
        generated::Body::Unknown => Err(structural_error()),
        generated::Body::Event { .. } => Err(ProtocolError::WrongMessageType(EVENT_MESSAGE)),
    }
}
pub(super) fn encode_event(value: &Event) -> Result<Vec<u8>, ProtocolError> {
    let kind = value.payload.event_kind();
    validate_event(value).map_err(|error| error.at(format!("body.event(type={kind:?})")))?;
    encode_envelope(generated::Body::Event {
        surface_id: Some(value.meta.surface_id),
        epoch: Some(value.meta.epoch),
        revision: Some(value.meta.revision),
        sequence: Some(value.meta.sequence),
        node_id: Some(value.meta.node_id),
        listener_id: Some(value.meta.listener_id),
        event_type: Some(wire_event_kind(kind.into())?),
        payload: wire_event_payload(&value.payload),
    })
}

fn checked_size(left: usize, right: usize) -> Result<usize, ProtocolError> {
    left.checked_add(right)
        .ok_or(ProtocolError::FrameTooLarge(usize::MAX))
}
fn field_size(value: usize) -> Result<usize, ProtocolError> {
    checked_size(1, value)
}
fn record_size(fields: &[usize]) -> Result<usize, ProtocolError> {
    let mut size = 5;
    for field in fields {
        size = checked_size(size, field_size(*field)?)?;
    }
    Ok(size)
}
fn string_size(value: &str) -> Result<usize, ProtocolError> {
    checked_size(4, value.len())
}
fn bytes_size(value: &[u8]) -> Result<usize, ProtocolError> {
    checked_size(4, value.len())
}
fn string_array_size(values: &[String]) -> Result<usize, ProtocolError> {
    let mut size = 4;
    for value in values {
        size = checked_size(size, string_size(value)?)?;
    }
    Ok(size)
}
fn union_size(value: usize) -> Result<usize, ProtocolError> {
    checked_size(5, value)
}
fn command_value_size(value: &CommandValue) -> Result<usize, ProtocolError> {
    let value = match value {
        CommandValue::Number(_) => record_size(&[4])?,
        CommandValue::Pair(_) => record_size(&[4, 4])?,
        CommandValue::Bool(_) => record_size(&[1])?,
        CommandValue::Text(value) => record_size(&[string_size(value)?])?,
        CommandValue::Paths(values) => record_size(&[string_array_size(values)?])?,
        CommandValue::FileText(value) => record_size(&[string_size(value)?])?,
        CommandValue::Image(value) => record_size(&[4, bytes_size(&value.bytes)?])?,
        CommandValue::Bounds(_) => record_size(&[4, 4, 4, 4])?,
        CommandValue::WindowState(_) => record_size(&[1, 1])?,
        CommandValue::ScrollOffset(_) => record_size(&[4])?,
        CommandValue::Bytes(value) => record_size(&[bytes_size(value)?])?,
    };
    union_size(value)
}
fn event_payload_size(value: &EventPayload) -> Result<usize, ProtocolError> {
    let value = match value {
        EventPayload::ApplicationActivation { reason, urls, .. } => {
            record_size(&[4, string_size(reason)?, string_array_size(urls)?])?
        }
        EventPayload::TextInputChange(value)
        | EventPayload::TextInputSelection(value)
        | EventPayload::FocusTextInput(value)
        | EventPayload::BlurTextInput(value) => {
            let mut fields = [0; 7];
            let mut count = 0;
            fields[count] = string_size(&value.text)?;
            count += 1;
            fields[count] = 4;
            count += 1;
            fields[count] = 4;
            count += 1;
            if value.marked_start.is_some() {
                fields[count] = 4;
                count += 1;
            }
            if value.marked_end.is_some() {
                fields[count] = 4;
                count += 1;
            }
            fields[count] = 4;
            count += 1;
            fields[count] = 1;
            count += 1;
            record_size(&fields[..count])?
        }
        EventPayload::CommandResult(value) => {
            let mut fields = [0; 6];
            let mut count = 4;
            fields[0] = 4;
            fields[1] = 4;
            fields[2] = 4;
            fields[3] = 1;
            if let Some(error) = &value.error {
                fields[count] = string_size(error)?;
                count += 1;
            }
            if let Some(value) = &value.value {
                fields[count] = command_value_size(value)?;
                count += 1;
            }
            record_size(&fields[..count])?
        }
        EventPayload::Press
        | EventPayload::Focus
        | EventPayload::Blur
        | EventPayload::Hover
        | EventPayload::SurfaceClosed => return Err(ProtocolError::InvalidEventPayload),
        EventPayload::VisibleRange { .. } => record_size(&[4, 4])?,
        EventPayload::AnimationComplete { .. } => record_size(&[4])?,
        EventPayload::Key(value) => record_size(&[
            string_size(&value.key)?,
            string_array_size(&value.modifiers)?,
            4,
        ])?,
        EventPayload::Pointer(value) => {
            record_size(&[4, string_array_size(&value.modifiers)?, 4, 4, 4, 4])?
        }
        EventPayload::PointerMove(value) => {
            record_size(&[string_array_size(&value.modifiers)?, 4, 4])?
        }
        EventPayload::Scroll(value) => {
            record_size(&[4, 4, 4, 4, 4, string_array_size(&value.modifiers)?])?
        }
        EventPayload::Submit { text } => record_size(&[string_size(text)?])?,
        EventPayload::WindowResize { .. } => record_size(&[4, 4, 4])?,
        EventPayload::WindowActivation { .. } => record_size(&[1])?,
        EventPayload::EventAction { action } => record_size(&[string_size(action)?])?,
        EventPayload::WindowAppearance { .. } => record_size(&[1])?,
        EventPayload::Layout { .. } => record_size(&[4, 4, 4, 4])?,
        EventPayload::DragOver { drag_type } | EventPayload::DragDrop { drag_type } => {
            record_size(&[string_size(drag_type)?])?
        }
        EventPayload::ExternalFileDrop { paths } => record_size(&[string_array_size(paths)?])?,
        EventPayload::NotificationResponse(value) => {
            let mut fields = [0; 2];
            let mut count = 1;
            fields[0] = string_size(&value.tag)?;
            if let Some(action_id) = &value.action_id {
                fields[count] = string_size(action_id)?;
                count += 1;
            }
            record_size(&fields[..count])?
        }
        EventPayload::PointerDownOutside { .. } => record_size(&[4, 4])?,
        EventPayload::CloseRequested { .. } => record_size(&[4])?,
        EventPayload::Extension { fields, .. } => {
            let fields_size = fields.iter().try_fold(4usize, |size, field| {
                let value_size = match &field.value {
                    ExtensionValue::Bool(_) => record_size(&[1])?,
                    ExtensionValue::Int32(_) | ExtensionValue::U32(_) | ExtensionValue::F32(_) => {
                        record_size(&[4])?
                    }
                    ExtensionValue::Text(value) => record_size(&[string_size(value)?])?,
                    ExtensionValue::Bytes(value) => record_size(&[bytes_size(value)?])?,
                };
                checked_size(size, record_size(&[4, union_size(value_size)?])?)
            })?;
            record_size(&[4, fields_size])?
        }
    };
    union_size(value)
}
pub(super) fn event_frame_size(value: &Event) -> Result<usize, ProtocolError> {
    validate_event(value)?;
    let event_payload = match &value.payload {
        EventPayload::Press
        | EventPayload::Focus
        | EventPayload::Blur
        | EventPayload::Hover
        | EventPayload::SurfaceClosed => None,
        _ => Some(event_payload_size(&value.payload)?),
    };
    let mut fields = [0; 8];
    fields[..6].fill(4);
    fields[6] = 1;
    let field_count = if let Some(payload) = event_payload {
        fields[7] = payload;
        8
    } else {
        7
    };
    let event = record_size(&fields[..field_count])?;
    let body = union_size(event)?;
    let envelope = record_size(&[4, body])?;
    let frame = checked_size(4, envelope)?;
    if envelope > MAX_FRAME_LENGTH {
        return Err(ProtocolError::FrameTooLarge(envelope));
    }
    Ok(frame)
}
pub(super) fn encode_event_frame(value: &Event, output: &mut Vec<u8>) -> Result<(), ProtocolError> {
    let kind = value.payload.event_kind();
    let body = generated::Body::Event {
        surface_id: Some(value.meta.surface_id),
        epoch: Some(value.meta.epoch),
        revision: Some(value.meta.revision),
        sequence: Some(value.meta.sequence),
        node_id: Some(value.meta.node_id),
        listener_id: Some(value.meta.listener_id),
        event_type: Some(wire_event_kind(kind.into())?),
        payload: wire_event_payload(&value.payload),
    };
    let envelope = generated::Envelope {
        protocol_version: Some(PROTOCOL_VERSION),
        body: Some(body),
    };
    let payload_length = envelope.serialized_size();
    if payload_length > MAX_FRAME_LENGTH {
        return Err(ProtocolError::FrameTooLarge(payload_length));
    }
    output.clear();
    output.resize(4, 0);
    envelope.serialize(output).map_err(ProtocolError::Encode)?;
    let length = u32::try_from(output.len() - 4)
        .map_err(|_| ProtocolError::FrameTooLarge(output.len() - 4))?;
    output[..4].copy_from_slice(&length.to_le_bytes());
    Ok(())
}

pub(super) fn classify_payload(
    payload: &[u8],
) -> Result<super::super::PayloadClassification, ProtocolError> {
    let body = decode_envelope(payload)?
        .body
        .ok_or_else(structural_error)?;
    match body {
        generated::Body::Snapshot { .. } => Ok((SNAPSHOT_MESSAGE, None, None, None, None)),
        generated::Body::Patch { .. } => Ok((PATCH_MESSAGE, None, None, None, None)),
        generated::Body::Command {
            kind, request_id, ..
        } => Ok((
            COMMAND_MESSAGE,
            None,
            kind.map(command_kind_value),
            request_id,
            None,
        )),
        generated::Body::Event {
            event_type,
            payload,
            ..
        } => {
            let (request_id, success) = match payload {
                Some(generated::EventPayload::CommandResult {
                    request_id,
                    success,
                    ..
                }) => (request_id, success),
                _ => (None, None),
            };
            Ok((
                EVENT_MESSAGE,
                event_type.map(event_kind_value),
                None,
                request_id,
                success,
            ))
        }
        generated::Body::Unknown => Err(structural_error()),
    }
}
