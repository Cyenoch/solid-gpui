use std::io::Cursor;

use super::node::{
    AccessibilityWire, HostPropertiesWire, NodeWire, StyleWire, valid_tooltip_text,
    validate_style_wire,
};
use super::*;
use serde::{Deserialize, Serialize};

pub(super) fn encode_snapshot(snapshot: &Snapshot) -> Result<Vec<u8>, ProtocolError> {
    let wire = SnapshotWire(
        snapshot.protocol,
        snapshot.message,
        snapshot.surface_id,
        snapshot.epoch,
        snapshot.base_revision,
        snapshot.revision,
        snapshot.nodes.iter().map(NodeWire::from).collect(),
    );
    rmp_serde::to_vec(&wire).map_err(ProtocolError::Encode)
}

pub(super) fn decode_snapshot(payload: &[u8]) -> Result<Snapshot, ProtocolError> {
    let mut deserializer = rmp_serde::Deserializer::new(Cursor::new(payload));
    let wire = SnapshotWire::deserialize(&mut deserializer).map_err(ProtocolError::Decode)?;
    let consumed = deserializer.into_inner().position() as usize;
    if consumed != payload.len() {
        return Err(ProtocolError::TrailingBytes(payload.len() - consumed));
    }
    if wire.0 != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedProtocol {
            received: wire.0,
            expected: PROTOCOL_VERSION,
        });
    }
    if wire.1 != SNAPSHOT_MESSAGE {
        return Err(ProtocolError::WrongMessageType(wire.1));
    }
    Ok(Snapshot {
        protocol: wire.0,
        message: wire.1,
        surface_id: wire.2,
        epoch: wire.3,
        base_revision: wire.4,
        revision: wire.5,
        nodes: wire
            .6
            .into_iter()
            .map(Node::try_from)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub(super) fn encode_patch(patch: &Patch) -> Result<Vec<u8>, ProtocolError> {
    let wire = PatchWire(
        patch.protocol,
        patch.message,
        patch.surface_id,
        patch.epoch,
        patch.base_revision,
        patch.revision,
        patch.operations.iter().map(OperationWire::from).collect(),
    );
    rmp_serde::to_vec(&wire).map_err(ProtocolError::Encode)
}

pub(super) fn decode_patch(payload: &[u8]) -> Result<Patch, ProtocolError> {
    let mut deserializer = rmp_serde::Deserializer::new(Cursor::new(payload));
    let wire = PatchWire::deserialize(&mut deserializer).map_err(ProtocolError::Decode)?;
    let consumed = deserializer.into_inner().position() as usize;
    if consumed != payload.len() {
        return Err(ProtocolError::TrailingBytes(payload.len() - consumed));
    }
    if wire.0 != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedProtocol {
            received: wire.0,
            expected: PROTOCOL_VERSION,
        });
    }
    if wire.1 != PATCH_MESSAGE {
        return Err(ProtocolError::WrongMessageType(wire.1));
    }
    Ok(Patch {
        protocol: wire.0,
        message: wire.1,
        surface_id: wire.2,
        epoch: wire.3,
        base_revision: wire.4,
        revision: wire.5,
        operations: wire
            .6
            .into_iter()
            .map(PatchOperation::try_from)
            .collect::<Result<Vec<_>, _>>()?,
    })
}
#[derive(Debug, Serialize, Deserialize)]
struct SnapshotWire(u32, u32, u32, u32, u32, u32, Vec<NodeWire>);
#[derive(Debug, Serialize, Deserialize)]
struct PatchWire(u32, u32, u32, u32, u32, u32, Vec<OperationWire>);
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OperationWire {
    Create(CreateWire),
    Update(UpdateWire),
    Move(MoveWire),
    Delete(DeleteWire),
}
#[derive(Debug, Serialize, Deserialize)]
struct CreateWire(
    u32,
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
struct UpdateWire(
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
struct MoveWire(u32, u32, u32, u32);

#[derive(Debug, Serialize, Deserialize)]
struct DeleteWire(u32, u32);

impl From<&PatchOperation> for OperationWire {
    fn from(operation: &PatchOperation) -> Self {
        match operation {
            PatchOperation::Create(node) => Self::Create(CreateWire(
                1,
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
            )),
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
            } => Self::Update(UpdateWire(
                2,
                *id,
                *mask,
                style.as_ref().map(StyleWire::from),
                text.clone(),
                *listener_id,
                host_properties.as_ref().map(HostPropertiesWire::from),
                accessibility.as_ref().map(AccessibilityWire::from),
                *focusable,
                (*selectable || tooltip.is_some()).then_some(*selectable),
                tooltip.clone(),
            )),
            PatchOperation::Move {
                id,
                parent_id,
                index,
            } => Self::Move(MoveWire(3, *id, *parent_id, *index)),
            PatchOperation::Delete { id } => Self::Delete(DeleteWire(4, *id)),
        }
    }
}

impl TryFrom<OperationWire> for PatchOperation {
    type Error = ProtocolError;

    fn try_from(operation: OperationWire) -> Result<Self, Self::Error> {
        match operation {
            OperationWire::Create(wire) => {
                if wire.0 != 1 {
                    return Err(ProtocolError::UnknownPatchOperation(wire.0));
                }
                if let Some(style) = wire.5.as_ref() {
                    validate_style_wire(style)?;
                }
                let selectable = wire.11.unwrap_or(false);
                if selectable && wire.4 != crate::tree::KIND_TEXT {
                    return Err(ProtocolError::InvalidHostProperties);
                }
                let host_properties = wire.8.map(HostProperties::try_from).transpose()?;
                let tooltip = wire.12;
                if tooltip
                    .as_ref()
                    .is_some_and(|value| !valid_tooltip_text(value))
                {
                    return Err(ProtocolError::InvalidHostProperties);
                }
                Ok(Self::Create(Node {
                    id: wire.1,
                    parent_id: wire.2,
                    index: wire.3,
                    kind: wire.4,
                    style: wire.5.map(Style::from),
                    text: wire.6,
                    listener_id: wire.7,
                    host_properties,
                    accessibility: wire.9.map(AccessibilityProperties::from),
                    focusable: wire.10,
                    selectable,
                    tooltip,
                }))
            }
            OperationWire::Update(wire) => {
                if wire.0 != 2 {
                    return Err(ProtocolError::UnknownPatchOperation(wire.0));
                }
                if let Some(style) = wire.3.as_ref() {
                    validate_style_wire(style)?;
                }
                let selectable = wire.9.unwrap_or(false);
                let tooltip = wire.10;
                if tooltip
                    .as_ref()
                    .is_some_and(|value| !valid_tooltip_text(value))
                {
                    return Err(ProtocolError::InvalidHostProperties);
                }
                let host_properties = wire.6.map(HostProperties::try_from).transpose()?;
                Ok(Self::Update {
                    id: wire.1,
                    mask: wire.2,
                    style: wire.3.map(Style::from),
                    text: wire.4,
                    listener_id: wire.5,
                    host_properties,
                    accessibility: wire.7.map(AccessibilityProperties::from),
                    focusable: wire.8,
                    selectable,
                    tooltip,
                })
            }
            OperationWire::Move(wire) => {
                if wire.0 != 3 {
                    return Err(ProtocolError::UnknownPatchOperation(wire.0));
                }
                Ok(Self::Move {
                    id: wire.1,
                    parent_id: wire.2,
                    index: wire.3,
                })
            }
            OperationWire::Delete(wire) => {
                if wire.0 != 4 {
                    return Err(ProtocolError::UnknownPatchOperation(wire.0));
                }
                Ok(Self::Delete { id: wire.1 })
            }
        }
    }
}
