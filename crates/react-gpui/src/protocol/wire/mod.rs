use super::*;

mod command;
mod event;
mod node;
mod snapshot_patch;

pub(super) fn decode_command(payload: &[u8]) -> Result<Command, ProtocolError> {
    command::decode_command(payload)
}

pub(super) fn encode_command(command: &Command) -> Result<Vec<u8>, ProtocolError> {
    command::encode_command(command)
}

pub(super) fn decode_event(payload: &[u8]) -> Result<Event, ProtocolError> {
    event::decode_event(payload)
}

pub(super) fn encode_event(event: &Event) -> Result<Vec<u8>, ProtocolError> {
    event::encode_event(event)
}

pub(super) fn decode_patch(payload: &[u8]) -> Result<Patch, ProtocolError> {
    snapshot_patch::decode_patch(payload)
}

pub(super) fn encode_patch(patch: &Patch) -> Result<Vec<u8>, ProtocolError> {
    snapshot_patch::encode_patch(patch)
}

pub(super) fn decode_snapshot(payload: &[u8]) -> Result<Snapshot, ProtocolError> {
    snapshot_patch::decode_snapshot(payload)
}

pub(super) fn encode_snapshot(snapshot: &Snapshot) -> Result<Vec<u8>, ProtocolError> {
    snapshot_patch::encode_snapshot(snapshot)
}
