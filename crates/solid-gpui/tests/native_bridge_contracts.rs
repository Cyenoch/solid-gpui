use solid_gpui::{
    ExtensionRegistry,
    native::{
        CommandDefinition, MAX_NATIVE_CALL_BYTES, ModuleDefinition, decode_json, encode_json,
    },
    native_type,
};

#[native_type]
struct Nested {
    values: Vec<Option<Choice>>,
}

#[native_type]
enum Choice {
    Value(String),
    Empty,
}

#[test]
fn nested_dtos_use_strict_dispatch_and_preserve_unknown_field_errors() {
    let definition = ModuleDefinition::new(
        "echo",
        vec![],
        vec![CommandDefinition::sync(
            "echo",
            |value: Nested, _context| Ok(value),
        )],
    );
    let module = definition
        .native_module(definition.id(), definition.digest())
        .unwrap();
    let input = br#"{"values":[{"Value":"hello"},null,"Empty"]}"#;
    assert_eq!(module.invoke(1, input).unwrap(), input);
    assert!(module.invoke(2, input).is_err());
    for invalid in [
        br#"{"values":[],"unexpected":1}"#.as_slice(),
        br#"{"values":[]} true"#.as_slice(),
        br#"{"values":[],"values":[]}"#.as_slice(),
    ] {
        assert!(module.invoke(1, invalid).is_err());
    }
    assert!(
        definition
            .typescript()
            .unwrap()
            .contains("Array<Choice | null>")
    );
}

#[test]
fn values_reject_json_precision_loss_and_resource_overflow() {
    assert!(encode_json(&f64::NAN).is_err());
    assert!(encode_json(&u64::MAX).is_err());
    assert!(encode_json(&9_007_199_254_740_992_f64).is_err());
    assert!(encode_json(&"x".repeat(MAX_NATIVE_CALL_BYTES)).is_err());
    assert!(decode_json::<()>(&vec![b' '; MAX_NATIVE_CALL_BYTES + 1]).is_err());
    assert!(decode_json::<serde_json::Value>(b"9007199254740992").is_err());
    let definition = ModuleDefinition::new(
        "errors",
        vec![],
        vec![CommandDefinition::sync("fail", |(): (), _context| {
            Err::<(), _>("domain error".into())
        })],
    );
    assert_eq!(
        definition
            .native_module(definition.id(), definition.digest())
            .unwrap()
            .invoke(1, b"null")
            .unwrap_err(),
        "domain error"
    );
}

#[test]
fn exported_contract_rejects_bigint_and_digest_tracks_the_signature() {
    let large = ModuleDefinition::new(
        "large",
        vec![],
        vec![CommandDefinition::sync("large", |(): (), _context| {
            Ok(u64::MAX)
        })],
    );
    assert!(large.typescript().is_err());
    let original = ModuleDefinition::new(
        "typed",
        vec![],
        vec![CommandDefinition::sync("get", |(): (), _context| {
            Ok(String::new())
        })],
    );
    let changed = ModuleDefinition::new(
        "typed",
        vec![],
        vec![CommandDefinition::sync("get", |(): (), _context| Ok(false))],
    );
    assert_eq!(original.id(), changed.id());
    assert_ne!(original.digest(), changed.digest());
}

#[test]
fn ambiguous_type_and_command_names_cannot_form_a_contract() {
    #[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
    #[ts(rename = "SharedName")]
    struct TextValue {
        value: String,
    }
    #[derive(serde::Serialize, ts_rs::TS)]
    #[ts(rename = "SharedName")]
    struct BoolValue {
        value: bool,
    }
    assert!(
        std::panic::catch_unwind(|| {
            ModuleDefinition::new(
                "type-conflict",
                vec![],
                vec![CommandDefinition::sync(
                    "convert",
                    |_: TextValue, _context| Ok(BoolValue { value: true }),
                )],
            )
        })
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            ModuleDefinition::new(
                "command-conflict",
                vec![],
                vec![
                    CommandDefinition::sync("same", |(): (), _context| Ok(false)),
                    CommandDefinition::sync("same", |(): (), _context| Ok(true)),
                ],
            )
        })
        .is_err()
    );
}

#[test]
fn composed_modules_export_shared_multiline_types_as_valid_typescript() {
    use solid_gpui::native::NativeModules;
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    #[native_type]
    struct SharedDto {
        /// A field documented over multiple
        /// lines by the Rust author.
        value: String,
    }
    let make = |namespace, command| {
        ModuleDefinition::new(
            namespace,
            vec![],
            vec![CommandDefinition::sync(
                command,
                |value: SharedDto, _context| Ok(value),
            )],
        )
    };
    let source = NativeModules::new(vec![make("one", "first"), make("two", "second")])
        .typescript()
        .unwrap();
    assert_eq!(source.matches("export type SharedDto").count(), 1);
    let mut parser = Command::new("bun")
        .args([
            "-e",
            "new Bun.Transpiler({loader:'ts'}).transformSync(await Bun.stdin.text());",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    parser
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let output = parser.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "generated TypeScript failed to parse: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[solid_gpui::native_module]
mod cancellable_service {
    use solid_gpui::native::NativeCallContext;

    #[command]
    fn echo(value: String, context: NativeCallContext) -> Result<String, String> {
        context.check_cancelled()?;
        Ok(value)
    }
}

#[test]
fn authored_context_is_injected_without_entering_the_wire_contract() {
    let definition = cancellable_service::native_module();
    let source = definition.typescript().unwrap();
    assert!(!source.contains("context:"));
    assert!(source.contains("options?: NativeCallOptions"));
    let module = definition
        .native_module(definition.id(), definition.digest())
        .unwrap();
    assert_eq!(
        module.invoke(1, br#"{"value":"ready"}"#).unwrap(),
        br#""ready""#
    );
}
