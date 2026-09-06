use super::*;
mod adapter;

pub(super) fn decode_command(payload: &[u8]) -> Result<super::Command, super::ProtocolError> {
    adapter::decode_command(payload)
}

pub(super) fn encode_command(command: &super::Command) -> Result<Vec<u8>, super::ProtocolError> {
    adapter::encode_command(command)
}
pub(super) fn decode_message(
    payload: &[u8],
) -> Result<super::DecodedMessage, super::ProtocolError> {
    adapter::decode_message(payload)
}

pub(super) fn decode_event(payload: &[u8]) -> Result<super::Event, super::ProtocolError> {
    adapter::decode_event(payload)
}

pub(super) fn encode_event(event: &super::Event) -> Result<Vec<u8>, super::ProtocolError> {
    adapter::encode_event(event)
}

pub(super) fn decode_patch(payload: &[u8]) -> Result<super::Patch, super::ProtocolError> {
    adapter::decode_patch(payload)
}

pub(super) fn encode_patch(patch: &super::Patch) -> Result<Vec<u8>, super::ProtocolError> {
    adapter::encode_patch(patch)
}

pub(super) fn decode_snapshot(payload: &[u8]) -> Result<super::Snapshot, super::ProtocolError> {
    adapter::decode_snapshot(payload)
}

pub(super) fn encode_snapshot(snapshot: &super::Snapshot) -> Result<Vec<u8>, super::ProtocolError> {
    adapter::encode_snapshot(snapshot)
}

pub(super) fn encode_event_frame(
    event: &super::Event,
    output: &mut Vec<u8>,
) -> Result<(), super::ProtocolError> {
    adapter::encode_event_frame(event, output)
}

pub(super) fn event_frame_size(event: &super::Event) -> Result<usize, super::ProtocolError> {
    adapter::event_frame_size(event)
}

pub(super) fn classify_payload(
    payload: &[u8],
) -> Result<super::PayloadClassification, super::ProtocolError> {
    adapter::classify_payload(payload)
}
