use solid_gpui::{
    ExtensionRegistry,
    native::{
        CommandDefinition, MAX_NATIVE_CALL_BYTES, ModuleDefinition, decode_json, encode_json,
        encode_native_request,
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
        "1.0.0",
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
    let envelope = |json: &[u8]| {
        let mut bytes = encode_native_request(definition.build_digest(), &()).unwrap();
        bytes.truncate(36);
        bytes.extend_from_slice(json);
        bytes
    };
    assert_eq!(module.invoke(1, &envelope(input)).unwrap(), input);
    assert!(module.invoke(2, &envelope(input)).is_err());
    for invalid in [
        br#"{"values":[],"unexpected":1}"#.as_slice(),
        br#"{"values":[]} true"#.as_slice(),
        br#"{"values":[],"values":[]}"#.as_slice(),
    ] {
        assert!(module.invoke(1, &envelope(invalid)).is_err());
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
        "1.0.0",
        vec![],
        vec![CommandDefinition::sync("fail", |(): (), _context| {
            Err::<(), _>("domain error".into())
        })],
    );
    assert_eq!(
        definition
            .native_module(definition.id(), definition.digest())
            .unwrap()
            .invoke(
                1,
                &encode_native_request(definition.build_digest(), &()).unwrap()
            )
            .unwrap_err(),
        "domain error"
    );
}

#[test]
fn exported_contract_rejects_bigint_and_digest_tracks_the_signature() {
    let large = ModuleDefinition::new(
        "large",
        "1.0.0",
        vec![],
        vec![CommandDefinition::sync("large", |(): (), _context| {
            Ok(u64::MAX)
        })],
    );
    assert!(large.typescript().is_err());
    let original = ModuleDefinition::new(
        "typed",
        "1.0.0",
        vec![],
        vec![CommandDefinition::sync("get", |(): (), _context| {
            Ok(String::new())
        })],
    );
    let changed = ModuleDefinition::new(
        "typed",
        "1.0.0",
        vec![],
        vec![CommandDefinition::sync("get", |(): (), _context| Ok(false))],
    );
    assert_eq!(original.id(), changed.id());
    assert_ne!(original.digest(), changed.digest());
}

#[test]
fn source_formatting_is_not_a_contract_change_and_stale_builds_are_rejected() {
    use solid_gpui::native::{ComponentDefinition, encode_native_request};
    let make = |source| {
        ModuleDefinition::new(
            "identity",
            "1.0.0",
            vec![
                ComponentDefinition::element("Panel", vec![], |_: &(), _| gpui::div())
                    .with_implementation(source),
            ],
            vec![CommandDefinition::sync("echo", |value: String, _| {
                Ok(value)
            })],
        )
        .with_semantic_version("1.0.0")
    };
    let original = make("fn render() {}\n");
    let commented = make("// Implementation note\nfn render() {}\n");
    let crlf = make("fn render() {}\r\n");
    assert_eq!(original.digest(), commented.digest());
    assert_eq!(original.digest(), crlf.digest());
    assert_ne!(original.build_digest(), commented.build_digest());
    assert_eq!(original.build_digest(), crlf.build_digest());
    let module = commented
        .native_module(original.id(), original.digest())
        .unwrap();
    let stale = encode_native_request(original.build_digest(), &"ready").unwrap();
    assert!(
        module
            .invoke(1, &stale)
            .unwrap_err()
            .contains("native build mismatch")
    );
    let current = encode_native_request(commented.build_digest(), &"ready").unwrap();
    assert_eq!(module.invoke(1, &current).unwrap(), br#""ready""#);
    assert!(
        module
            .invoke(1, br#""ready""#)
            .unwrap_err()
            .contains("native build envelope")
    );
    use solid_gpui::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
    use solid_gpui::{ExtensionChildSummary, ExtensionError};
    let props = |build| ExtensionProperties {
        provider_id: original.id(),
        catalog_digest: original.digest(),
        entry_id: 1,
        entry_version: 1,
        fields: vec![ExtensionField {
            id: 1,
            value: ExtensionValue::Bytes(encode_native_request(build, &()).unwrap()),
        }],
        event_ids: [].into(),
    };
    let adapter = commented
        .resolve(original.id(), original.digest(), 1, 1)
        .unwrap();
    assert!(matches!(
        adapter.validate(
            1,
            &props(original.build_digest()),
            ExtensionChildSummary::default()
        ),
        Err(ExtensionError::BuildMismatch { .. })
    ));
    adapter
        .validate(
            1,
            &props(commented.build_digest()),
            ExtensionChildSummary::default(),
        )
        .unwrap();
}

#[test]
fn exported_metadata_and_behavior_versions_change_contract_identity() {
    use solid_gpui::native::{ComponentDefinition, EventDefinition};
    let make = |slots, events, version| {
        ModuleDefinition::new(
            "metadata",
            "1.0.0",
            vec![
                ComponentDefinition::element("Panel", events, |_: &(), _| gpui::div())
                    .with_slots(slots),
            ],
            vec![],
        )
        .with_semantic_version(version)
    };
    let original = make(&[], vec![], "1.0.0");
    for changed in [
        make(&["footer"], vec![], "1.0.0"),
        make(&[], vec![EventDefinition::new::<String>("change")], "1.0.0"),
        make(&[], vec![], "1.0.1"),
    ] {
        assert_ne!(original.digest(), changed.digest());
        assert!(
            original
                .resolve(changed.id(), changed.digest(), 1, 1)
                .is_err()
        );
    }
    let versioned_component = ModuleDefinition::new(
        "metadata",
        "1.0.0",
        vec![
            ComponentDefinition::element("Panel", vec![], |_: &(), _| gpui::div())
                .with_semantic_version("2.0.0"),
        ],
        vec![],
    );
    assert_ne!(original.digest(), versioned_component.digest());
    let included = original.include(ModuleDefinition::new("behavior", "2.0.0", vec![], vec![]));
    assert_ne!(versioned_component.digest(), included.digest());
    assert!(included.contract().to_string().contains("2.0.0"));
    let text_event = make(&[], vec![EventDefinition::new::<String>("change")], "1.0.0");
    let bool_event = make(&[], vec![EventDefinition::new::<bool>("change")], "1.0.0");
    assert_ne!(text_event.digest(), bool_event.digest());

    #[derive(serde::Deserialize, serde::Serialize, ts_rs::TS)]
    #[ts(rename = "State")]
    struct TextState {
        value: String,
    }
    #[derive(serde::Deserialize, serde::Serialize, ts_rs::TS)]
    #[ts(rename = "State")]
    struct BoolState {
        value: bool,
    }
    let text_dto = ModuleDefinition::new(
        "dto",
        "1.0.0",
        vec![],
        vec![CommandDefinition::sync("echo", |value: TextState, _| {
            Ok(value)
        })],
    );
    let bool_dto = ModuleDefinition::new(
        "dto",
        "1.0.0",
        vec![],
        vec![CommandDefinition::sync("echo", |value: BoolState, _| {
            Ok(value)
        })],
    );
    assert_ne!(text_dto.digest(), bool_dto.digest());
}

#[test]
fn dto_documentation_is_exported_without_changing_the_contract() {
    #[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
    #[ts(rename = "Documented")]
    struct First {
        /// Original documentation.
        text: String,
    }
    #[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
    #[ts(rename = "Documented")]
    struct Revised {
        /// Updated documentation with a literal /* marker.
        text: String,
    }
    let first = ModuleDefinition::new(
        "docs",
        "1.0.0",
        vec![],
        vec![CommandDefinition::sync("echo", |value: First, _| Ok(value))],
    );
    let revised = ModuleDefinition::new(
        "docs",
        "1.0.0",
        vec![],
        vec![CommandDefinition::sync("echo", |value: Revised, _| {
            Ok(value)
        })],
    );
    assert_eq!(first.digest(), revised.digest());
    assert_ne!(first.typescript().unwrap(), revised.typescript().unwrap());
}

#[test]
fn registry_distinguishes_missing_modules_from_mismatched_native_contracts() {
    use solid_gpui::{
        ExtensionError,
        native::{ComponentDefinition, NativeModules},
    };
    let definition = ModuleDefinition::new(
        "controls",
        "1.0.0",
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
                "1.0.0",
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
                "1.0.0",
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
            "1.0.0",
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

#[solid_gpui::native_module(version = "1.0.0")]
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
        module
            .invoke(
                1,
                &encode_native_request(
                    definition.build_digest(),
                    &serde_json::json!({"value":"ready"})
                )
                .unwrap()
            )
            .unwrap(),
        br#""ready""#
    );
}

#[solid_gpui::native_module(version = "1.0.0")]
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

#[solid_gpui::native_module(version = "1.0.0")]
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

#[solid_gpui::native_module(version = "1.0.0")]
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
