#[path = "generated/schema_guard.rs"]
mod schema_guard;

use super::ProtocolError;

#[derive(Clone, Copy, Debug)]
struct Limits {
    max_frame_bytes: usize,
    max_message_bytes: usize,
    max_array_items: usize,
    max_string_bytes: usize,
    max_bytes: usize,
    max_nesting: usize,
    max_total_items: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_frame_bytes: super::MAX_FRAME_LENGTH,
            max_message_bytes: super::MAX_FRAME_LENGTH,
            max_array_items: 100_000,
            max_string_bytes: super::MAX_CLIPBOARD_TEXT_BYTES,
            max_bytes: super::MAX_CLIPBOARD_IMAGE_BYTES,
            max_nesting: 80,
            max_total_items: 1_000_000,
        }
    }
}

struct Guard<'a> {
    data: &'a [u8],
    offset: usize,
    limits: Limits,
    depth: usize,
    total_items: usize,
    bounds: Vec<usize>,
}
impl<'a> Guard<'a> {
    fn end(&self) -> usize {
        self.bounds.last().copied().unwrap_or(self.data.len())
    }
    fn ensure(&self, size: usize) -> Result<(), String> {
        self.offset
            .checked_add(size)
            .filter(|end| *end <= self.end())
            .map(|_| ())
            .ok_or_else(|| format!("remaining-byte budget exceeded at byte {}", self.offset))
    }
    fn byte(&mut self) -> Result<u8, String> {
        self.ensure(1)?;
        let value = self.data[self.offset];
        self.offset += 1;
        Ok(value)
    }
    fn u32(&mut self) -> Result<u32, String> {
        self.ensure(4)?;
        let value = u32::from_le_bytes(
            self.data[self.offset..self.offset + 4]
                .try_into()
                .expect("guarded u32"),
        );
        self.offset += 4;
        Ok(value)
    }
    fn blob(&mut self, max: usize, label: &str) -> Result<(), String> {
        let length_offset = self.offset;
        let length = self.u32()? as usize;
        if length > max {
            return Err(format!(
                "{label} budget exceeded ({length} > {max}) at byte {length_offset}"
            ));
        }
        self.ensure(length)?;
        if label == "string" {
            std::str::from_utf8(&self.data[self.offset..self.offset + length])
                .map_err(|_| format!("invalid UTF-8 string at byte {}", self.offset))?;
        }
        self.offset += length;
        Ok(())
    }
    fn scalar(&mut self, name: &str, parent: &str, field: &str) -> Result<Option<i64>, String> {
        if name == "string" {
            self.blob(self.string_limit(parent, field), "string")?;
            return Ok(None);
        }
        if name == "bool" {
            let value = self.byte()?;
            if value > 1 {
                return Err(format!(
                    "boolean must be 0 or 1 at byte {}",
                    self.offset - 1
                ));
            }
            return Ok(Some(i64::from(value)));
        }
        let (size, signed) = match name {
            "byte" | "uint8" => (1, false),
            "uint16" => (2, false),
            "int16" => (2, true),
            "uint32" => (4, false),
            "int32" => (4, true),
            "float32" => (4, false),
            "uint64" => (8, false),
            "int64" => (8, true),
            "float64" | "date" => (8, false),
            "guid" => (16, false),
            other => return Err(format!("unknown scalar {other} at byte {}", self.offset)),
        };
        self.ensure(size)?;
        let start = self.offset;
        self.offset += size;
        if size == 1 {
            return Ok(Some(i64::from(self.data[start])));
        }
        if size == 2 {
            let value = u16::from_le_bytes(self.data[start..start + 2].try_into().unwrap());
            return Ok(Some(if signed {
                i64::from(i16::from_le_bytes(value.to_le_bytes()))
            } else {
                i64::from(value)
            }));
        }
        if size == 4 && signed {
            return Ok(Some(i64::from(i32::from_le_bytes(
                self.data[start..start + 4].try_into().unwrap(),
            ))));
        }
        if size == 4 {
            let bytes: [u8; 4] = self.data[start..start + 4].try_into().unwrap();
            return Ok(Some(i64::from(u32::from_le_bytes(bytes))));
        }
        Ok(None)
    }
    fn string_limit(&self, parent: &str, field: &str) -> usize {
        match (parent, field) {
            (_, "content") => super::MAX_FILE_WRITE_BYTES,
            (_, "fontFamily") => 256,
            (_, "path") => 1024,
            (_, "source" | "fallbackSource") => super::MAX_IMAGE_SOURCE_BYTES,
            (_, "dragType") => 512,
            (_, "tag") => 1024,
            (_, "title" | "action" | "name" | "label" | "key") => 1024,
            ("FileTextValue", "value") => super::MAX_FILE_READ_BYTES,
            _ => self.limits.max_string_bytes,
        }
    }
    fn array_limit(&self, field: &str) -> usize {
        match field {
            "modifiers" => 5,
            "actions" | "bindings" => 64,
            "values" => 2,
            "items" => 1024,
            "menus" => 64,
            "paths" | "exportFiles" => 4096,
            _ => self.limits.max_array_items,
        }
    }
    fn type_ref(&mut self, type_id: usize, parent: &str, field: &str) -> Result<(), String> {
        let type_spec = schema_guard::TYPES
            .get(type_id)
            .ok_or_else(|| format!("unknown schema type {type_id} at byte {}", self.offset))?;
        match type_spec {
            schema_guard::TypeSpec::Scalar(name) => {
                self.scalar(name, parent, field)?;
                Ok(())
            }
            schema_guard::TypeSpec::Definition { definition } => self.definition(*definition),
            schema_guard::TypeSpec::Array { element } => {
                let count_offset = self.offset;
                let count = self.u32()? as usize;
                if matches!(
                    schema_guard::TYPES.get(*element),
                    Some(schema_guard::TypeSpec::Scalar("byte"))
                ) {
                    let limit = match (parent, field) {
                        ("InvokeNativeCommand", "moduleId") => 16,
                        ("InvokeNativeCommand", "moduleDigest") => 32,
                        ("InvokeNativeCommand", "args") | ("BytesValue", "value") => {
                            super::MAX_NATIVE_CALL_BYTES
                        }
                        _ => self.limits.max_bytes,
                    };
                    if count > limit {
                        return Err(format!(
                            "bytes budget exceeded ({count} > {limit}) at byte {count_offset}"
                        ));
                    }
                    self.ensure(count)?;
                    self.offset += count;
                    return Ok(());
                }
                let limit = self.array_limit(field);
                if count > limit {
                    return Err(format!(
                        "array item budget exceeded ({count} > {limit}) at byte {count_offset}"
                    ));
                }
                self.total_items = self.total_items.checked_add(count).ok_or_else(|| {
                    format!("total array item count overflow at byte {count_offset}")
                })?;
                if self.total_items > self.limits.max_total_items {
                    return Err(format!(
                        "total array item budget exceeded at byte {count_offset}"
                    ));
                }
                for _ in 0..count {
                    self.type_ref(*element, parent, field)?;
                }
                Ok(())
            }
        }
    }
    fn definition(&mut self, definition_id: usize) -> Result<(), String> {
        self.depth += 1;
        if self.depth > self.limits.max_nesting {
            return Err(format!(
                "nesting budget exceeded ({}) at byte {}",
                self.limits.max_nesting, self.offset
            ));
        }
        let definition = schema_guard::DEFINITIONS
            .get(definition_id)
            .ok_or_else(|| {
                format!(
                    "unknown schema definition {definition_id} at byte {}",
                    self.offset
                )
            })?;
        let result = match definition {
            schema_guard::Definition::Enum { name, base, values } => {
                let offset = self.offset;
                let value = self.scalar(base, name, "")?;
                if let Some(value) = value
                    && !values.contains(&value)
                {
                    return Err(format!("unknown enum value {value} at byte {offset}"));
                }
                Ok(())
            }
            schema_guard::Definition::Union { branches } => {
                let length_offset = self.offset;
                let length = self.u32()? as usize;
                if length > self.limits.max_message_bytes {
                    return Err(format!(
                        "union budget exceeded ({length} > {}) at byte {length_offset}",
                        self.limits.max_message_bytes
                    ));
                }
                let end = self
                    .offset
                    .checked_add(1 + length)
                    .ok_or("union length overflow")?;
                self.ensure(1 + length)?;
                self.bounds.push(end);
                let tag_offset = self.offset;
                let tag = self.byte()?;
                let branch = branches
                    .iter()
                    .find(|branch| branch.id == tag)
                    .ok_or_else(|| {
                        format!("unknown union discriminator {tag} at byte {tag_offset}")
                    })?;
                self.definition(branch.definition)?;
                if self.offset != end {
                    return Err(format!(
                        "union payload has trailing bytes at byte {}",
                        self.offset
                    ));
                }
                self.bounds.pop();
                Ok(())
            }
            schema_guard::Definition::Message { name, fields } => {
                let length_offset = self.offset;
                let length = self.u32()? as usize;
                if length > self.limits.max_message_bytes {
                    return Err(format!(
                        "message budget exceeded ({length} > {}) at byte {length_offset}",
                        self.limits.max_message_bytes
                    ));
                }
                let end = self
                    .offset
                    .checked_add(length)
                    .ok_or("message length overflow")?;
                self.ensure(length)?;
                self.bounds.push(end);
                let mut last = 0;
                let mut terminated = false;
                while self.offset < end {
                    let id_offset = self.offset;
                    let id = self.byte()?;
                    if id == 0 {
                        terminated = true;
                        if self.offset != end {
                            return Err(format!(
                                "message bytes after terminator at byte {}",
                                self.offset
                            ));
                        }
                        break;
                    }
                    if id <= last {
                        let reason = if id == last {
                            "duplicate message field"
                        } else {
                            "message fields out of order"
                        };
                        return Err(format!("{reason} at byte {}", self.offset - 1));
                    }
                    last = id;
                    let field = fields
                        .binary_search_by_key(&id, |field| field.id)
                        .ok()
                        .map(|index| &fields[index])
                        .ok_or_else(|| format!("unknown message field {id} at byte {id_offset}"))?;
                    self.type_ref(field.type_id, name, field.name)?;
                }
                self.bounds.pop();
                if !terminated || self.offset != end {
                    return Err(format!(
                        "message terminator or consumed length missing at byte {}",
                        self.offset
                    ));
                }
                Ok(())
            }
        };
        self.depth -= 1;
        result
    }
    fn run(mut self) -> Result<(), String> {
        if self.data.len() > self.limits.max_frame_bytes {
            return Err(format!(
                "frame budget exceeded ({} > {}) at byte 0",
                self.data.len(),
                self.limits.max_frame_bytes
            ));
        }
        self.definition(schema_guard::ROOT_DEFINITION)?;
        Ok(())
    }
}

pub(super) fn guard(payload: &[u8]) -> Result<(), ProtocolError> {
    Guard {
        data: payload,
        offset: 0,
        limits: Limits::default(),
        depth: 0,
        total_items: 0,
        bounds: Vec::new(),
    }
    .run()
    .map_err(ProtocolError::BoundedDecode)
}
