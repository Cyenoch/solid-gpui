use super::executor::NativeExecutor;
use super::*;
use crate::{ExtensionAdapter, ExtensionRegistry};
use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, future::Future, sync::Arc};

type Handler =
    dyn Fn(Vec<u8>, &NativeExecutor) -> BoxFuture<'static, Result<Vec<u8>, String>> + Send + Sync;
#[derive(Clone)]
pub struct CommandDefinition {
    name: &'static str,
    describe: fn(&mut Types) -> (String, String),
    handler: Arc<Handler>,
}
impl CommandDefinition {
    /// Run synchronous work on Tokio's blocking pool. Cancelling its request
    /// discards the result, but an already-running closure must return on its own.
    /// Such work retains its global admission permit until it actually finishes.
    pub fn sync<I: DeserializeOwned + TS + Send + 'static, O: Serialize + TS + 'static>(
        name: &'static str,
        function: fn(I) -> Result<O, String>,
    ) -> Self {
        Self {
            name,
            describe: |types| (types.collect::<I>(), types.collect::<O>()),
            handler: Arc::new(move |bytes, executor| {
                executor.blocking(move || {
                    let request = decode_json(&bytes)?;
                    encode_json(&function(request)?)
                })
            }),
        }
    }
    /// Run the entire command future inside the shared Tokio runtime, including
    /// creation of timer/I/O futures. Dropping the invocation future aborts it.
    pub fn asynchronous<I, O, F, Fut>(name: &'static str, function: F) -> Self
    where
        I: DeserializeOwned + TS + Send + 'static,
        O: Serialize + TS + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, String>> + Send + 'static,
    {
        let function = Arc::new(function);
        Self {
            name,
            describe: |types| (types.collect::<I>(), types.collect::<O>()),
            handler: Arc::new(move |bytes, executor| {
                let function = Arc::clone(&function);
                executor.asynchronous(async move {
                    let request = decode_json(&bytes)?;
                    encode_json(&function(request).await?)
                })
            }),
        }
    }
}

#[derive(Clone)]
struct Commands {
    id: [u8; 16],
    digest: [u8; 32],
    entries: Vec<CommandDefinition>,
    executor: Arc<NativeExecutor>,
}
impl NativeModule for Commands {
    fn module_id(&self) -> [u8; 16] {
        self.id
    }
    fn module_digest(&self) -> [u8; 32] {
        self.digest
    }
    fn invoke(&self, id: u32, args: &[u8]) -> Result<Vec<u8>, String> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(
                "blocking native invocation is not allowed inside Tokio; await invoke_async".into(),
            );
        }
        futures::executor::block_on(self.invoke_async(id, args.to_vec()))
    }
    fn invoke_async(&self, id: u32, args: Vec<u8>) -> BoxFuture<'_, Result<Vec<u8>, String>> {
        match self.entries.get(id.wrapping_sub(1) as usize) {
            Some(command) => (command.handler)(args, &self.executor),
            None => Box::pin(async { Err("unknown native function".into()) }),
        }
    }
}

/// The exact module linked into the host also generates its JavaScript contract.
pub struct ModuleDefinition {
    name: String,
    id: [u8; 16],
    digest: [u8; 32],
    components: Vec<ComponentDefinition>,
    commands: Arc<Commands>,
    source: &'static str,
}
impl ModuleDefinition {
    pub fn new(
        name: &str,
        mut components: Vec<ComponentDefinition>,
        mut commands: Vec<CommandDefinition>,
    ) -> Self {
        components.sort_by_key(|c| c.name);
        commands.sort_by_key(|c| c.name);
        assert_eq!(
            components
                .iter()
                .map(|c| c.name)
                .collect::<BTreeSet<_>>()
                .len(),
            components.len(),
            "duplicate native component"
        );
        assert_eq!(
            commands
                .iter()
                .map(|c| c.name)
                .collect::<BTreeSet<_>>()
                .len(),
            commands.len(),
            "duplicate native command"
        );
        let hash = Sha256::digest(name.as_bytes());
        let mut id = [0; 16];
        id.copy_from_slice(&hash[..16]);
        let mut module = Self {
            name: name.into(),
            id,
            digest: [0; 32],
            components,
            commands: Arc::new(Commands {
                id,
                digest: [0; 32],
                entries: commands,
                executor: Arc::new(NativeExecutor::default()),
            }),
            source: "",
        };
        module.refresh_digest();
        module
    }
    pub fn with_contract(mut self, source: &'static str) -> Self {
        self.source = source;
        self.refresh_digest();
        self
    }
    pub fn with_component(mut self, component: ComponentDefinition) -> Self {
        assert!(
            !self
                .components
                .iter()
                .any(|entry| entry.name == component.name),
            "duplicate native component"
        );
        self.components.push(component);
        self.components.sort_by_key(|entry| entry.name);
        self.refresh_digest();
        self
    }
    pub fn id(&self) -> [u8; 16] {
        self.id
    }
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }
    fn description(&self) -> (Types, serde_json::Value) {
        let mut types = Types::default();
        let components=self.components.iter().enumerate().map(|(i,c)| {
            let props=(c.props)(&mut types);
            let events=c.events.iter().enumerate().map(|(i,e)|serde_json::json!({"id":i+1,"name":e.name,"prop":format!("on{}",pascal(e.name)),"type":(e.describe)(&mut types)})).collect::<Vec<_>>();
            let commands=c.commands.iter().enumerate().map(|(i,(name,describe))|{let(input,output)=describe(&mut types);serde_json::json!({"id":i+1,"name":name,"input":input,"output":output})}).collect::<Vec<_>>();
            serde_json::json!({"entryId":i+1,"entryVersion":1,"name":c.name,"propsType":props,"props":c.prop_names,"events":events,"commands":commands,"children":c.children,"controlled":c.controlled,"source":c.source})
        }).collect::<Vec<_>>();
        let commands = self
            .commands
            .entries
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let (input, output) = (c.describe)(&mut types);
                serde_json::json!({"id":i+1,"name":c.name,"input":input,"output":output})
            })
            .collect::<Vec<_>>();
        let value = serde_json::json!({"format":1,"module":self.name,"source":self.source,"types":types.declarations,"components":components,"commands":commands});
        (types, value)
    }
    fn refresh_digest(&mut self) {
        let (_, description) = self.description();
        self.digest = Sha256::digest(serde_json::to_vec(&description).unwrap()).into();
        Arc::get_mut(&mut self.commands)
            .expect("module is frozen after sharing")
            .digest = self.digest;
    }
    pub fn typescript(&self) -> Result<String, String> {
        self.typescript_named("", true)
    }
    fn typescript_named(&self, suffix: &str, include_types: bool) -> Result<String, String> {
        let (types, value) = self.description();
        types.check()?;
        let mut out = String::new();
        if include_types {
            out.push_str(TYPESCRIPT_HEADER);
            write_types(&mut out, &types.declarations);
        }
        for component in value["components"].as_array().unwrap() {
            let name = component["name"].as_str().unwrap();
            let wire_props = component["propsType"].as_str().unwrap();
            let props = match component["controlled"]["ackProp"].as_str() {
                Some(ack) => format!(
                    "Omit<{wire_props}, {}>",
                    serde_json::to_string(ack).unwrap()
                ),
                None => wire_props.to_owned(),
            };
            let mut events = String::new();
            for event in component["events"].as_array().unwrap() {
                let prop = event["prop"].as_str().unwrap();
                let ty = event["type"].as_str().unwrap();
                events.push_str(&format!(
                    "{prop}?: ({} ) => void;",
                    if ty == "null" {
                        String::new()
                    } else {
                        format!("value: {ty}")
                    }
                ));
            }
            let methods = methods_type(&component["commands"]);
            out.push_str(&format!("export type {name}Ref = {{ {methods} }};\n"));
            let mut descriptor = component.clone();
            let object = descriptor.as_object_mut().unwrap();
            for key in ["source", "name", "propsType"] {
                object.remove(key);
            }
            for event in descriptor["events"].as_array_mut().unwrap() {
                event.as_object_mut().unwrap().remove("type");
            }
            for command in descriptor["commands"].as_array_mut().unwrap() {
                let object = command.as_object_mut().unwrap();
                object.remove("input");
                object.remove("output");
            }
            descriptor["providerId"] = serde_json::json!(self.id);
            descriptor["catalogDigest"] = serde_json::json!(self.digest);
            out.push_str(&format!("export const {name} = createNativeComponent<{props}, {{ {events} }}, {name}Ref>({descriptor});\n"));
        }
        let descriptor = serde_json::json!({"moduleId":self.id,"moduleDigest":self.digest,"commands":value["commands"]});
        out.push_str(&format!(
            "export interface NativeClient{suffix} {{ {} }}\n",
            methods_type(&value["commands"])
        ));
        out.push_str(&format!("const descriptor{suffix} = {descriptor};\nexport const useNative{suffix} = () => useNativeClient<NativeClient{suffix}>(descriptor{suffix});\nexport const createClient{suffix} = (root: Parameters<typeof createNativeClient>[0]) => createNativeClient<NativeClient{suffix}>(root, descriptor{suffix});\n"));
        Ok(out)
    }
}
const TYPESCRIPT_HEADER: &str = "// Generated by the running Rust host. Do not edit.\nimport { createNativeComponent, createNativeClient, useNativeClient } from '@solid-gpui/core/native';\n";

fn write_types(output: &mut String, declarations: &std::collections::BTreeMap<String, String>) {
    for declaration in declarations.values() {
        output.push_str("export ");
        output.push_str(declaration);
        output.push('\n');
    }
}

fn methods_type(value: &serde_json::Value) -> String {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            format!(
                "{}({}): Promise<{}>;",
                c["name"].as_str().unwrap(),
                if c["input"] == "null" {
                    String::new()
                } else {
                    format!("request: {}", c["input"].as_str().unwrap())
                },
                c["output"].as_str().unwrap()
            )
        })
        .collect()
}
fn pascal(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
impl ExtensionRegistry for ModuleDefinition {
    fn resolve(
        &self,
        provider: [u8; 16],
        digest: [u8; 32],
        entry: u32,
        version: u32,
    ) -> Option<&dyn ExtensionAdapter> {
        if provider != self.id || digest != self.digest || version != 1 {
            return None;
        }
        self.components
            .get(entry.wrapping_sub(1) as usize)
            .map(|c| c as &dyn ExtensionAdapter)
    }
    fn native_module(&self, id: [u8; 16], digest: [u8; 32]) -> Option<Arc<dyn NativeModule>> {
        (id == self.id && digest == self.digest)
            .then(|| self.commands.clone() as Arc<dyn NativeModule>)
    }
}

/// Explicit module composition. The same collection supplies host registration and exports.
///
/// All modules share a lazily started Tokio runtime with timer and I/O drivers.
/// At most 128 asynchronous and blocking commands may be in flight across the
/// collection; saturation returns an error instead of queueing unbounded work.
/// Registered command handles keep the executor alive. Once its last owner drops,
/// shutdown starts without blocking GPUI. Async tasks are cancelled; a blocking
/// closure already running continues until it returns and retains its resources.
/// Do not detach child tasks from a command if they must share its cancellation.
/// Production UI code must await `NativeModule::invoke_async`; the blocking
/// `invoke` entry is for synchronous test/worker callers, never GPUI foreground.
pub struct NativeModules {
    modules: Vec<ModuleDefinition>,
}
impl NativeModules {
    pub fn new(mut modules: Vec<ModuleDefinition>) -> Self {
        assert_eq!(
            modules.iter().map(|m| m.id).collect::<BTreeSet<_>>().len(),
            modules.len(),
            "duplicate native module namespace"
        );
        let executor = Arc::new(NativeExecutor::default());
        for module in &mut modules {
            Arc::make_mut(&mut module.commands).executor = Arc::clone(&executor);
        }
        Self { modules }
    }
    pub fn typescript_modules(&self) -> Result<Vec<(&str, String)>, String> {
        self.modules
            .iter()
            .map(|m| Ok((m.name.as_str(), m.typescript()?)))
            .collect()
    }
    pub fn typescript(&self) -> Result<String, String> {
        let mut names = BTreeSet::new();
        let mut types = std::collections::BTreeMap::new();
        let mut exports = String::new();
        for (i, module) in self.modules.iter().enumerate() {
            for component in &module.components {
                if !names.insert(format!("component:{}", component.name)) {
                    return Err(format!("duplicate native export {}", component.name));
                }
            }
            for command in &module.commands.entries {
                if !names.insert(format!("command:{}", command.name)) {
                    return Err(format!("duplicate native command {}", command.name));
                }
            }
            for (name, decl) in module.description().0.declarations {
                if let Some(previous) = types.insert(name.clone(), decl.clone())
                    && previous != decl
                {
                    return Err(format!("conflicting native type {name}"));
                }
            }
            exports.push_str(&module.typescript_named(&i.to_string(), false)?);
        }
        let mut output = String::from(TYPESCRIPT_HEADER);
        write_types(&mut output, &types);
        output.push_str(&exports);
        let clients = (0..self.modules.len())
            .map(|i| format!("NativeClient{i}"))
            .collect::<Vec<_>>()
            .join(" & ");
        output.push_str(&format!(
            "export type NativeClient = {};\n",
            if clients.is_empty() { "{}" } else { &clients }
        ));
        output.push_str(&format!(
            "export const useNative = (): NativeClient => ({{ {} }});\n",
            (0..self.modules.len())
                .map(|i| format!("...useNative{i}()"))
                .collect::<Vec<_>>()
                .join(",")
        ));
        output.push_str(&format!("export const createClient = (root: Parameters<typeof createNativeClient>[0]): NativeClient => ({{ {} }});\n",(0..self.modules.len()).map(|i|format!("...createClient{i}(root)")).collect::<Vec<_>>().join(",")));
        Ok(output)
    }
}
impl ExtensionRegistry for NativeModules {
    fn resolve(&self, p: [u8; 16], d: [u8; 32], e: u32, v: u32) -> Option<&dyn ExtensionAdapter> {
        self.modules.iter().find_map(|m| m.resolve(p, d, e, v))
    }
    fn native_module(&self, id: [u8; 16], digest: [u8; 32]) -> Option<Arc<dyn NativeModule>> {
        self.modules
            .iter()
            .find_map(|m| m.native_module(id, digest))
    }
}
