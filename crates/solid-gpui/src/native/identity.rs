use super::{Serialize, encode_json};
use sha2::{Digest, Sha256};

const REQUEST_HEADER: &[u8; 4] = b"SGN\x02";

/// Encode the strict native build envelope used for props and invocation arguments.
/// Results and events remain typed JSON; admission has already checked the build.
pub fn encode_native_request<T: Serialize + ?Sized>(
    build_digest: [u8; 32],
    value: &T,
) -> Result<Vec<u8>, String> {
    let json = encode_json(value)?;
    if json.len() > super::MAX_NATIVE_CALL_BYTES - 36 {
        return Err("native build envelope exceeds 1 MiB".into());
    }
    let mut bytes = Vec::with_capacity(36 + json.len());
    bytes.extend_from_slice(REQUEST_HEADER);
    bytes.extend_from_slice(&build_digest);
    bytes.extend_from_slice(&json);
    Ok(bytes)
}

pub(super) fn request_json(bytes: &[u8], expected: [u8; 32]) -> Result<&[u8], String> {
    let json = validated_request_json(bytes)?;
    if bytes[4..36] != expected {
        return Err(format!(
            "native build mismatch: renderer build {}, host build {}; regenerate bindings and rebuild the application with the selected host",
            hex(&bytes[4..36]),
            hex(&expected),
        ));
    }
    Ok(json)
}

/// Only for already validated committed props, including typed child projections.
pub(crate) fn validated_request_json(bytes: &[u8]) -> Result<&[u8], String> {
    if bytes.len() > super::MAX_NATIVE_CALL_BYTES {
        return Err("native build envelope exceeds 1 MiB".into());
    }
    if bytes.len() < 36 || &bytes[..4] != REQUEST_HEADER {
        return Err("native build envelope is missing or unsupported; regenerate bindings from the selected host".into());
    }
    Ok(&bytes[36..])
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(super) fn source_digest(source: &str) -> [u8; 32] {
    Sha256::digest(source.replace("\r\n", "\n").as_bytes()).into()
}

pub(super) fn semantic_version(version: &str) {
    assert!(
        version.split('.').count() == 3
            && version.split('.').all(|part| {
                !part.is_empty()
                    && (part == "0" || !part.starts_with('0'))
                    && part.bytes().all(|byte| byte.is_ascii_digit())
                    && part.parse::<u32>().is_ok()
            }),
        "native semantic version must be an explicit major.minor.patch version"
    );
}

/// Canonicalize ts-rs declarations while retaining literal contents. Documentation
/// is exported separately and must not become part of compatibility identity.
pub(super) fn canonical_type(source: &str) -> String {
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(character) = chars.next() {
        if character == '/' && matches!(chars.peek(), Some('/' | '*')) {
            if chars.next() == Some('/') {
                for character in chars.by_ref() {
                    if matches!(character, '\r' | '\n') {
                        break;
                    }
                }
            } else {
                let mut previous = '\0';
                for character in chars.by_ref() {
                    if previous == '*' && character == '/' {
                        break;
                    }
                    previous = character;
                }
            }
            continue;
        }
        if character.is_whitespace() {
            continue;
        }
        let mut token = character.to_string();
        if matches!(character, '\'' | '"' | '`') {
            while let Some(next) = chars.next() {
                token.push(next);
                if next == '\\' {
                    if let Some(escaped) = chars.next() {
                        token.push(escaped);
                    }
                } else if next == character {
                    break;
                }
            }
        } else if character.is_alphanumeric() || matches!(character, '_' | '$') {
            while chars
                .peek()
                .is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '$'))
            {
                token.push(chars.next().unwrap());
            }
        }
        tokens.push(token);
    }
    serde_json::to_string(&tokens).unwrap()
}
