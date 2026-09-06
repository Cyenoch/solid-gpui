extern crate self as solid_gpui;
pub use solid_gpui_macros::{component, native_module, native_type};

pub mod gpui {
    pub struct AnyElement;
    pub trait IntoElement {
        fn into_any_element(self) -> AnyElement;
    }
    impl IntoElement for AnyElement {
        fn into_any_element(self) -> AnyElement {
            self
        }
    }
}

pub mod native {
    pub use serde;
    use serde::{Serialize, de::DeserializeOwned};
    use std::{future::Future, marker::PhantomData};
    pub use ts_rs;
    use ts_rs::TS;

    pub struct Event<T>(PhantomData<T>);
    impl<T> Event<T> {
        pub fn emit(&self, _: T) {}
    }
    pub struct EventDefinition;
    impl EventDefinition {
        pub fn new<T: Serialize + TS>(_: &'static str) -> Self {
            Self
        }
    }
    pub struct ElementContext<'a>(PhantomData<&'a ()>);
    impl ElementContext<'_> {
        pub fn event<T>(&self, _: &str) -> Event<T> {
            Event(PhantomData)
        }
    }
    pub trait NativeView {}
    pub struct ComponentDefinition(pub String, pub bool);
    impl ComponentDefinition {
        pub fn element<P: Clone + DeserializeOwned + TS>(
            _: &'static str,
            _: Vec<EventDefinition>,
            _: fn(&P, &mut ElementContext<'_>) -> crate::gpui::AnyElement,
        ) -> Self {
            Self(P::decl(&ts_rs::Config::new()), true)
        }
        pub fn with_children(mut self, enabled: bool) -> Self {
            self.1 = enabled;
            self
        }
        pub fn with_props(self, _: &'static [&'static str]) -> Self {
            self
        }
        pub fn view<T: NativeView>(_: &'static str) -> Self {
            Self(String::new(), false)
        }
        pub fn with_contract(self, contract: &'static str) -> Self {
            assert!(!contract.is_empty());
            self
        }
    }
    pub struct CommandDefinition(pub String);
    impl CommandDefinition {
        pub fn sync<I: DeserializeOwned + TS, O: Serialize + TS>(
            _: &'static str,
            _: fn(I) -> Result<O, String>,
        ) -> Self {
            Self(I::name(&ts_rs::Config::new()))
        }
        pub fn asynchronous<
            I: DeserializeOwned + TS,
            O: Serialize + TS,
            F: Fn(I) -> Fut,
            Fut: Future<Output = Result<O, String>>,
        >(
            _: &'static str,
            _: F,
        ) -> Self {
            Self(I::name(&ts_rs::Config::new()))
        }
    }
    pub struct ModuleDefinition(pub Vec<ComponentDefinition>, pub Vec<CommandDefinition>);
    impl ModuleDefinition {
        pub fn new(
            _: &str,
            components: Vec<ComponentDefinition>,
            commands: Vec<CommandDefinition>,
        ) -> Self {
            Self(components, commands)
        }
        pub fn with_contract(self, contract: &'static str) -> Self {
            assert!(!contract.is_empty());
            self
        }
    }
}

#[native_module(name = "test")]
mod app {
    use super::{gpui, native::*, native_type};

    #[native_type]
    #[derive(Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Settings {
        pub display_name: String,
        #[serde(default)]
        pub enabled: bool,
        pub note: Option<String>,
    }

    #[component(children = false)]
    fn greeting(
        settings: Settings,
        cx: &mut ElementContext,
        on_press: Event<()>,
        #[prop(default = true)] disabled: bool,
    ) -> impl gpui::IntoElement {
        let _ = (settings, cx, disabled);
        on_press.emit(());
        gpui::AnyElement
    }

    struct Editor;
    #[component]
    impl NativeView for Editor {}

    #[cfg(any())]
    #[command]
    fn excluded_command(value: MissingType) -> MissingType {
        value
    }

    #[cfg(any())]
    #[component]
    fn excluded_component(value: MissingType) -> MissingType {
        value
    }

    #[command]
    fn version() -> String {
        "1".into()
    }

    #[command]
    fn count(settings: Settings) -> u32 {
        settings.display_name.len() as u32
    }

    #[command]
    async fn load(#[prop(default)] path: String) -> Result<Settings, std::io::Error> {
        Ok(Settings {
            display_name: path,
            enabled: false,
            note: None,
        })
    }
}

#[test]
fn exports_compile_with_reexports_and_default_wire_shapes() {
    let module = app::native_module();
    assert_eq!(module.0.len(), 2);
    assert!(!module.0[0].1);
    assert_eq!(module.1.len(), 3);
    assert_eq!(module.1[0].0, "null");
    assert!(
        module.0[0].0.contains("disabled?: boolean"),
        "{}",
        module.0[0].0
    );
    let declaration = <app::Settings as ts_rs::TS>::decl(&ts_rs::Config::new());
    assert!(declaration.contains("enabled?: boolean"), "{declaration}");
    assert!(
        declaration.contains("note?: string | null"),
        "{declaration}"
    );
    let settings: app::Settings = serde_json::from_str(r#"{"displayName":"test"}"#).unwrap();
    assert!(!settings.enabled);
    assert!(
        serde_json::from_str::<app::Settings>(r#"{"displayName":"test","unknown":1}"#).is_err()
    );
}
