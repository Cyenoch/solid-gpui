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
fn registry_distinguishes_missing_modules_from_mismatched_native_contracts() {
    use solid_gpui::{
        ExtensionError,
        native::{ComponentDefinition, NativeModules},
    };
    let definition = ModuleDefinition::new(
        "controls",
        vec![ComponentDefinition::element(
            "ScrollShadow",
            vec![],
            |_: &(), _| gpui::div(),
        )],
        vec![],
    );
    let id = definition.id();
    let digest = definition.digest();
    let registry = NativeModules::new(vec![definition]);
    assert!(registry.resolve(id, digest, 1, 1).is_ok());
    let error = registry.resolve(id, [0; 32], 1, 1).err().unwrap();
    assert!(
        matches!(&error, ExtensionError::ContractMismatch { module, .. } if module == "controls")
    );
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("renderer catalog"));
    assert!(diagnostic.contains("host catalog"));
    assert!(diagnostic.contains("host entry: ScrollShadow"));
    assert!(
        registry
            .resolve(id, digest, 1, 2)
            .err()
            .unwrap()
            .to_string()
            .contains("version 2 is unsupported")
    );
    assert!(
        registry
            .resolve(id, digest, 2, 1)
            .err()
            .unwrap()
            .to_string()
            .contains("entry 2 is not registered")
    );
    assert!(matches!(
        registry.resolve([0; 16], digest, 1, 1),
        Err(ExtensionError::AdapterNotFound { .. })
    ));
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
    assert_typescript_parses(&source);
}

fn assert_typescript_parses(source: &str) {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
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

#[solid_gpui::native_module]
mod documented_dtos {
    use solid_gpui::native_type;

    #[native_type]
    enum DesktopIntent {
        #[serde(rename = "any")]
        Open,
        #[serde(rename = "bigint")]
        Close,
    }

    #[native_type]
    struct DesktopState {
        /// The pending request, if any.
        pending: Option<DesktopIntent>,
        any: String,
        bigint: bool,
    }

    #[command]
    fn echo(state: DesktopState) -> DesktopState {
        state
    }
}

#[solid_gpui::native_module]
mod unsupported_any_dtos {
    use solid_gpui::native::{Deserialize, Serialize, TS};
    use solid_gpui::native_type;

    // A dependency can implement TS independently of the native_type macro.
    #[derive(Serialize, Deserialize, TS)]
    struct UnboundedValue(#[ts(type = "any")] String);

    #[native_type]
    struct AnyState {
        value: Option<Vec<UnboundedValue>>,
    }

    #[command]
    fn echo_any(state: AnyState) -> AnyState {
        state
    }
}

#[solid_gpui::native_module]
mod unsupported_bigint_dtos {
    use solid_gpui::native_type;

    #[native_type]
    struct BigintState {
        value: Option<Vec<u64>>,
    }
    #[command]
    fn echo_bigint(state: BigintState) -> BigintState {
        state
    }
}

#[test]
fn documented_native_dtos_export_complete_bindings_without_losing_documentation() {
    use solid_gpui::native::NativeModules;

    let definition = documented_dtos::native_module();
    let source = definition.typescript().unwrap();
    assert!(source.contains("The pending request, if any."));
    assert!(source.contains("any: string"));
    assert!(source.contains("bigint: boolean"));
    assert!(source.contains("\"any\" | \"bigint\""));
    assert!(source.contains("Promise<DesktopState>"));
    assert_typescript_parses(&source);

    let modules = NativeModules::new(vec![definition]);
    let complete = modules.typescript().unwrap();
    assert!(complete.contains("export type DesktopIntent"));
    assert!(complete.contains("export type DesktopState"));
    assert!(complete.contains("export const createClient ="));
    assert_typescript_parses(&complete);
    assert_eq!(modules.typescript_modules().unwrap()[0].1, source);
}

#[test]
fn unsupported_native_dtos_fail_individual_and_composed_exports() {
    use solid_gpui::native::NativeModules;

    for definition in [
        unsupported_any_dtos::native_module(),
        unsupported_bigint_dtos::native_module(),
    ] {
        let error = definition.typescript().unwrap_err();
        assert!(error.contains("native DTOs do not support bigint or any"));
        let modules = NativeModules::new(vec![documented_dtos::native_module(), definition]);
        assert_eq!(modules.typescript().unwrap_err(), error);
        assert_eq!(modules.typescript_modules().unwrap_err(), error);
    }
}
