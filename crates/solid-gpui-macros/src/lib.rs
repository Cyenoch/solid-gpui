use heck::{ToLowerCamelCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::{format_ident, quote};
use syn::{
    Attribute, Expr, FnArg, GenericArgument, Ident, Item, ItemFn, ItemImpl, ItemMod, LitStr, Meta,
    Pat, PathArguments, ReturnType, Signature, Type, parse_quote, punctuated::Punctuated,
    visit::Visit,
};

fn finish(result: syn::Result<Tokens>) -> TokenStream {
    result.unwrap_or_else(syn::Error::into_compile_error).into()
}

#[proc_macro_attribute]
pub fn native_type(attr: TokenStream, item: TokenStream) -> TokenStream {
    finish((|| {
        reject_options(attr.into())?;
        expand_native_type(syn::parse(item)?)
    })())
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    finish((|| {
        let (children, descriptor, validate) = component_options(attr.into())?;
        match syn::parse::<Item>(item)? {
            Item::Fn(function) => {
                expand_component(function, children.unwrap_or(true), descriptor, validate)
            }
            Item::Impl(implementation) => {
                if children == Some(true) || descriptor || validate.is_some() {
                    return Err(syn::Error::new_spanned(
                        implementation,
                        "retained NativeView components cannot accept JS children",
                    ));
                }
                expand_view(implementation)
            }
            item => Err(syn::Error::new_spanned(
                item,
                "component requires a function or NativeView impl",
            )),
        }
    })())
}

fn component_options(options: Tokens) -> syn::Result<(Option<bool>, bool, Option<Expr>)> {
    let mut children = None;
    let mut descriptor = false;
    let mut validate = None;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("validate") {
            if validate.is_some() {
                return Err(meta.error("duplicate validation option"));
            }
            validate = Some(meta.value()?.parse::<Expr>()?);
            return Ok(());
        }
        if meta.path.is_ident("descriptor") {
            if descriptor {
                return Err(meta.error("duplicate descriptor option"));
            }
            descriptor = true;
            return Ok(());
        }
        if !meta.path.is_ident("children") {
            return Err(meta.error(
                "unknown component option; expected descriptor, validate, or children = true or false",
            ));
        }
        if children.is_some() {
            return Err(meta.error("duplicate component children option"));
        }
        children = Some(meta.value()?.parse::<syn::LitBool>()?.value);
        Ok(())
    });
    syn::parse::Parser::parse2(parser, options)?;
    Ok((children, descriptor, validate))
}

fn qualify_component_attribute(attr: &mut Attribute) -> syn::Result<()> {
    match &mut attr.meta {
        Meta::Path(path) => *path = parse_quote!(::solid_gpui::component),
        Meta::List(list) => {
            component_options(list.tokens.clone())?;
            list.path = parse_quote!(::solid_gpui::component);
        }
        Meta::NameValue(_) => {
            return Err(syn::Error::new_spanned(
                attr,
                "component options require parentheses",
            ));
        }
    }
    Ok(())
}

#[proc_macro_attribute]
pub fn native_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    finish(expand_module(
        attr.into(),
        syn::parse_macro_input!(item as ItemMod),
    ))
}

fn reject_options(options: Tokens) -> syn::Result<()> {
    if options.is_empty() {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            options,
            "this attribute accepts no options",
        ))
    }
}

fn is_attr(attr: &Attribute, name: &str) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn add_derives(attrs: &mut Vec<Attribute>, names: &[&str]) -> syn::Result<()> {
    let mut existing = Vec::new();
    for attr in attrs.iter().filter(|attr| is_attr(attr, "derive")) {
        for path in
            attr.parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)?
        {
            if let Some(segment) = path.segments.last() {
                existing.push(segment.ident.to_string());
            }
        }
    }
    let paths: Vec<syn::Path> = names
        .iter()
        .filter(|name| !existing.iter().any(|old| old == **name))
        .map(|name| match *name {
            "Serialize" => parse_quote!(::solid_gpui::native::serde::Serialize),
            "Deserialize" => parse_quote!(::solid_gpui::native::serde::Deserialize),
            "TS" => parse_quote!(::solid_gpui::native::ts_rs::TS),
            "Clone" => parse_quote!(::core::clone::Clone),
            _ => unreachable!(),
        })
        .collect();
    if !paths.is_empty() {
        attrs.insert(0, parse_quote!(#[derive(#(#paths),*)]));
    }
    attrs.push(parse_quote!(#[serde(crate = "solid_gpui::native::serde")]));
    attrs.push(parse_quote!(#[ts(crate = "solid_gpui::native::ts_rs")]));
    Ok(())
}

fn validate_serde(attrs: &[Attribute]) -> syn::Result<()> {
    for attr in attrs {
        if is_attr(attr, "ts") {
            return Err(syn::Error::new_spanned(
                attr,
                "native_type derives its wire type; independent ts overrides are unsupported",
            ));
        }
        if !is_attr(attr, "serde") {
            continue;
        }
        for meta in attr.parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated)? {
            let name = meta
                .path()
                .get_ident()
                .map(ToString::to_string)
                .unwrap_or_default();
            let allowed = match &meta {
                Meta::Path(_) => matches!(
                    name.as_str(),
                    "untagged" | "default" | "deny_unknown_fields" | "transparent"
                ),
                Meta::NameValue(_) => matches!(
                    name.as_str(),
                    "rename" | "rename_all" | "rename_all_fields" | "tag" | "content" | "default"
                ),
                Meta::List(_) => false,
            };
            if !allowed {
                return Err(syn::Error::new_spanned(
                    meta,
                    "unsupported native wire attribute; use symmetric rename/tag/content/default without custom serializers, flatten or skipped fields",
                ));
            }
        }
    }
    Ok(())
}

fn has_serde_flag(attrs: &[Attribute], flag: &str) -> bool {
    attrs
        .iter()
        .filter(|attr| is_attr(attr, "serde"))
        .any(|attr| {
            attr.parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated)
                .is_ok_and(|items| items.iter().any(|meta| meta.path().is_ident(flag)))
        })
}

fn expand_native_type(mut item: Item) -> syn::Result<Tokens> {
    match &mut item {
        Item::Struct(item) => {
            validate_serde(&item.attrs)?;
            let container_default = has_serde_flag(&item.attrs, "default");
            for field in &mut item.fields {
                validate_serde(&field.attrs)?;
                if field.ident.is_some()
                    && (container_default || has_serde_flag(&field.attrs, "default"))
                {
                    field
                        .attrs
                        .push(parse_quote!(#[ts(as = "Option<_>", optional)]));
                } else if field.ident.is_some()
                    && type_last(&field.ty).is_some_and(|segment| segment.ident == "Option")
                {
                    field.attrs.push(parse_quote!(#[ts(optional = nullable)]));
                }
            }
            add_derives(&mut item.attrs, &["Serialize", "Deserialize", "TS"])?;
            if !has_serde_flag(&item.attrs, "deny_unknown_fields")
                && !has_serde_flag(&item.attrs, "transparent")
            {
                item.attrs.push(parse_quote!(#[serde(deny_unknown_fields)]));
            }
        }
        Item::Enum(item) => {
            validate_serde(&item.attrs)?;
            for variant in &mut item.variants {
                validate_serde(&variant.attrs)?;
                for field in &mut variant.fields {
                    validate_serde(&field.attrs)?;
                    if field.ident.is_some() && has_serde_flag(&field.attrs, "default") {
                        field
                            .attrs
                            .push(parse_quote!(#[ts(as = "Option<_>", optional)]));
                    } else if field.ident.is_some()
                        && type_last(&field.ty).is_some_and(|segment| segment.ident == "Option")
                    {
                        field.attrs.push(parse_quote!(#[ts(optional = nullable)]));
                    }
                }
            }
            add_derives(&mut item.attrs, &["Serialize", "Deserialize", "TS"])?;
        }
        _ => {
            return Err(syn::Error::new_spanned(
                item,
                "native_type requires a struct or enum",
            ));
        }
    }
    Ok(quote!(#item))
}

fn validate_signature(sig: &Signature, allow_async: bool) -> syn::Result<()> {
    if !sig.generics.params.is_empty() || sig.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "native exports cannot be generic",
        ));
    }
    if sig.unsafety.is_some()
        || sig.abi.is_some()
        || sig.variadic.is_some()
        || sig.constness.is_some()
        || (!allow_async && sig.asyncness.is_some())
    {
        return Err(syn::Error::new_spanned(
            sig,
            "unsupported native export signature",
        ));
    }
    Ok(())
}

fn parameter(argument: &mut FnArg) -> syn::Result<(Ident, Type, Vec<Attribute>)> {
    let FnArg::Typed(argument) = argument else {
        return Err(syn::Error::new_spanned(
            argument,
            "native export parameters cannot contain self",
        ));
    };
    let Pat::Ident(pattern) = &*argument.pat else {
        return Err(syn::Error::new_spanned(
            &argument.pat,
            "native parameters must be named identifiers",
        ));
    };
    if pattern.by_ref.is_some() || pattern.subpat.is_some() {
        return Err(syn::Error::new_spanned(
            pattern,
            "native parameters must be named identifiers",
        ));
    }
    let attrs = std::mem::take(&mut argument.attrs);
    Ok((pattern.ident.clone(), (*argument.ty).clone(), attrs))
}

fn type_last(ty: &Type) -> Option<&syn::PathSegment> {
    if let Type::Path(path) = ty {
        path.path.segments.last()
    } else {
        None
    }
}

fn event_type(ty: &Type) -> syn::Result<Option<Type>> {
    let Some(segment) = type_last(ty).filter(|segment| segment.ident == "Event") else {
        return Ok(None);
    };
    if let PathArguments::AngleBracketed(args) = &segment.arguments
        && args.args.len() == 1
        && let Some(GenericArgument::Type(ty)) = args.args.first()
    {
        return Ok(Some(ty.clone()));
    }
    Err(syn::Error::new_spanned(
        ty,
        "Event requires exactly one payload type",
    ))
}

fn is_context(ty: &Type) -> bool {
    matches!(ty, Type::Reference(reference) if reference.mutability.is_some() && type_last(&reference.elem).is_some_and(|segment| segment.ident == "ElementContext"))
}

fn owned_type(ty: &Type) -> syn::Result<()> {
    struct Check(Option<syn::Error>);
    impl<'ast> Visit<'ast> for Check {
        fn visit_type(&mut self, ty: &'ast Type) {
            if matches!(
                ty,
                Type::Reference(_)
                    | Type::Ptr(_)
                    | Type::BareFn(_)
                    | Type::ImplTrait(_)
                    | Type::TraitObject(_)
            ) {
                self.0 = Some(syn::Error::new_spanned(
                    ty,
                    "native payloads must be owned data types",
                ));
            } else {
                syn::visit::visit_type(self, ty);
            }
        }
    }
    let mut check = Check(None);
    check.visit_type(ty);
    check.0.map_or(Ok(()), Err)
}

fn field(
    name: &Ident,
    ty: &Type,
    attrs: Vec<Attribute>,
    prefix: &Ident,
) -> syn::Result<(Tokens, Tokens)> {
    owned_type(ty)?;
    let mut default: Option<Option<Expr>> = None;
    for attr in attrs {
        if !is_attr(&attr, "prop") {
            return Err(syn::Error::new_spanned(
                attr,
                "only prop(default) or prop(default = expression) is supported on native parameters",
            ));
        }
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("default") {
                return Err(meta.error("unknown prop option; expected default"));
            }
            if default.is_some() {
                return Err(meta.error("duplicate prop default"));
            }
            default = Some(if meta.input.peek(syn::Token![=]) {
                Some(meta.value()?.parse()?)
            } else {
                None
            });
            Ok(())
        })?;
        if default.is_none() {
            return Err(syn::Error::new_spanned(
                attr,
                "prop requires a default option",
            ));
        }
    }
    let wire_name = name
        .to_string()
        .trim_start_matches("r#")
        .to_lower_camel_case();
    let (default_attr, helper) = match default {
        None if type_last(ty).is_some_and(|segment| segment.ident == "Option") => {
            (quote!(#[ts(optional = nullable)]), quote!())
        }
        None => (quote!(), quote!()),
        Some(None) => (
            quote!(#[serde(default)] #[ts(as = "Option<_>", optional)]),
            quote!(),
        ),
        Some(Some(expr)) => {
            let helper = format_ident!(
                "__native_default_{}_{}",
                prefix,
                name.to_string().trim_start_matches("r#")
            );
            let path = helper.to_string();
            (
                quote!(#[serde(default = #path)] #[ts(as = "Option<_>", optional)]),
                quote!(fn #helper() -> #ty { #expr }),
            )
        }
    };
    Ok((
        quote!(#[serde(rename = #wire_name)] #default_attr #name: #ty),
        helper,
    ))
}

fn expand_component(
    mut function: ItemFn,
    children: bool,
    descriptor: bool,
    validate: Option<Expr>,
) -> syn::Result<Tokens> {
    let contract = quote!(#function #validate).to_string();
    // Native elements must be owned. Explicit capture avoids accidentally
    // retaining the borrowed render context in an opaque Rust 2024 return type.
    if let ReturnType::Type(_, ty) = &mut function.sig.output
        && let Type::ImplTrait(opaque) = ty.as_mut()
    {
        opaque.bounds.push(parse_quote!(use<>));
    }
    let output = function.sig.output.clone();
    let styled = matches!(&output, ReturnType::Type(_, ty) if matches!(ty.as_ref(), Type::ImplTrait(opaque) if opaque.bounds.iter().any(|b| matches!(b, syn::TypeParamBound::Trait(bound) if bound.path.segments.last().is_some_and(|s| s.ident == "Styled")))));
    let constructor = match (descriptor, styled) {
        (true, true) => format_ident!("styled_descriptor"),
        (true, false) => format_ident!("descriptor"),
        (false, true) => format_ident!("styled_element"),
        (false, false) => format_ident!("element"),
    };
    // Parameters declare the generated JSX props/events. Their arity is the
    // public component schema, not a handwritten positional call interface.
    function
        .attrs
        .push(parse_quote!(#[allow(clippy::too_many_arguments)]));
    validate_signature(&function.sig, false)?;
    let name = function.sig.ident.clone();
    let definition = format_ident!("__native_component_{}", name);
    let props = format_ident!("__NativeProps{}", name.to_string().to_upper_camel_case());
    let js_name = name.to_string().to_upper_camel_case();
    let mut fields = Vec::new();
    let mut helpers = Vec::new();
    let mut args = Vec::new();
    let mut events = Vec::new();
    let mut event_bindings = Vec::new();
    let mut prop_names = Vec::new();
    let mut slots = Vec::new();
    let mut child_type = None;
    let mut context_count = 0;
    for argument in &mut function.sig.inputs {
        let (name, ty, attrs) = parameter(argument)?;
        if is_context(&ty) {
            if !attrs.is_empty() {
                return Err(syn::Error::new_spanned(
                    &attrs[0],
                    "context parameters do not accept prop attributes",
                ));
            }
            context_count += 1;
            if context_count > 1 {
                return Err(syn::Error::new_spanned(
                    ty,
                    "only one ElementContext may be injected",
                ));
            }
            args.push(quote!(__cx));
        } else if let Type::Path(path) = &ty
            && let Some(segment) = path.path.segments.last()
            && segment.ident == "NativeItems"
        {
            if !attrs.is_empty() {
                return Err(syn::Error::new_spanned(
                    &attrs[0],
                    "typed children do not accept prop attributes",
                ));
            }
            if child_type.is_some() {
                return Err(syn::Error::new_spanned(
                    ty,
                    "only one NativeItems parameter may consume children",
                ));
            }
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return Err(syn::Error::new_spanned(
                    ty,
                    "NativeItems requires an element type",
                ));
            };
            let Some(GenericArgument::Type(item)) = arguments.args.first() else {
                return Err(syn::Error::new_spanned(
                    ty,
                    "NativeItems requires an element type",
                ));
            };
            child_type = Some(item.clone());
            args.push(quote!(__cx.typed_children::<#item>()));
        } else if matches!(&ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "NativeSlot"))
        {
            if !attrs.is_empty() {
                return Err(syn::Error::new_spanned(
                    &attrs[0],
                    "slots do not accept prop attributes",
                ));
            }
            let slot = name.to_string().to_lower_camel_case();
            slots.push(slot.clone());
            args.push(quote!(__cx.slot(#slot)));
        } else if let Some(payload) = event_type(&ty)? {
            owned_type(&payload)?;
            if !attrs.is_empty() {
                return Err(syn::Error::new_spanned(
                    &attrs[0],
                    "event parameters do not accept prop attributes",
                ));
            }
            let rust_name = name.to_string();
            let Some(event_name) = rust_name
                .strip_prefix("on_")
                .filter(|name| !name.is_empty())
            else {
                return Err(syn::Error::new_spanned(
                    name,
                    "Event parameters must be named on_event_name",
                ));
            };
            let event_name = event_name.to_lower_camel_case();
            events
                .push(quote!(::solid_gpui::native::EventDefinition::new::<#payload>(#event_name)));
            let local = format_ident!("__native_event_{}", name);
            event_bindings.push(quote!(let #local = __cx.event::<#payload>(#event_name);));
            args.push(quote!(#local));
        } else {
            let (field, helper) = field(&name, &ty, attrs, &function.sig.ident)?;
            fields.push(field);
            prop_names.push(
                name.to_string()
                    .trim_start_matches("r#")
                    .to_lower_camel_case(),
            );
            helpers.push(helper);
            args.push(quote!(__props.#name.clone()));
        }
    }
    let child_contract = child_type.map(|ty| quote!(.with_child_type::<#ty>()));
    let validation = validate.map(|v| quote!(.with_validation::<#props>(#v)));
    let mut attrs = Vec::new();
    add_derives(&mut attrs, &["Clone", "Deserialize", "TS"])?;
    Ok(quote! {
        #function
        #(#helpers)*
        #(#attrs)*
        #[serde(deny_unknown_fields)]
        struct #props { #(#fields),* }
        #[allow(non_snake_case)]
        fn #definition() -> ::solid_gpui::native::ComponentDefinition {
            fn render(__props: &#props, __cx: &mut ::solid_gpui::native::ElementContext<'_>) #output {
                #(#event_bindings)*
                #name(#(#args),*)
            }
            ::solid_gpui::native::ComponentDefinition::#constructor::<#props, _>(#js_name, vec![#(#events),*], render).with_contract(#contract).with_props(&[#(#prop_names),*]).with_children(#children).with_slots(&[#(#slots),*]) #child_contract #validation
        }
    })
}

fn view_name(implementation: &ItemImpl) -> syn::Result<Ident> {
    if !implementation.generics.params.is_empty() || implementation.generics.where_clause.is_some()
    {
        return Err(syn::Error::new_spanned(
            &implementation.generics,
            "native views cannot be generic",
        ));
    }
    let Some((None, path, _)) = &implementation.trait_ else {
        return Err(syn::Error::new_spanned(
            implementation,
            "component impl requires NativeView",
        ));
    };
    if !path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "NativeView")
    {
        return Err(syn::Error::new_spanned(
            path,
            "component impl requires NativeView",
        ));
    }
    let Some(segment) = type_last(&implementation.self_ty) else {
        return Err(syn::Error::new_spanned(
            &implementation.self_ty,
            "native view requires a named concrete type",
        ));
    };
    if !matches!(segment.arguments, PathArguments::None) {
        return Err(syn::Error::new_spanned(
            segment,
            "native view requires a non-generic type",
        ));
    }
    Ok(segment.ident.clone())
}

fn expand_view(implementation: ItemImpl) -> syn::Result<Tokens> {
    let contract = quote!(#implementation).to_string();
    let name = view_name(&implementation)?;
    let definition = format_ident!("__native_component_{}", name);
    let js_name = name.to_string().to_upper_camel_case();
    let ty = &implementation.self_ty;
    Ok(quote! {
        #implementation
        #[allow(non_snake_case)]
        fn #definition() -> ::solid_gpui::native::ComponentDefinition {
            ::solid_gpui::native::ComponentDefinition::view::<#ty>(#js_name).with_contract(#contract)
        }
    })
}

fn return_type(output: &ReturnType) -> (Type, bool) {
    let ty = match output {
        ReturnType::Default => parse_quote!(()),
        ReturnType::Type(_, ty) => (**ty).clone(),
    };
    if let Some(segment) = type_last(&ty).filter(|segment| segment.ident == "Result")
        && let PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(GenericArgument::Type(output)) = args.args.first()
    {
        return (output.clone(), true);
    }
    (ty, false)
}

fn expand_command(function: &mut ItemFn) -> syn::Result<(Tokens, Tokens)> {
    validate_signature(&function.sig, true)?;
    let name = function.sig.ident.clone();
    let request = format_ident!("__NativeRequest{}", name.to_string().to_upper_camel_case());
    let helper = format_ident!("__native_command_{}", name);
    let js_name = name.to_string().to_lower_camel_case();
    let mut fields = Vec::new();
    let mut helpers = Vec::new();
    let mut args = Vec::new();
    for argument in &mut function.sig.inputs {
        let (arg, ty, attrs) = parameter(argument)?;
        let (field, helper) = field(&arg, &ty, attrs, &name)?;
        fields.push(field);
        helpers.push(helper);
        args.push(quote!(__request.#arg));
    }
    let (output, fallible) = return_type(&function.sig.output);
    owned_type(&output)?;
    let call = if function.sig.asyncness.is_some() {
        quote!(#name(#(#args),*).await)
    } else {
        quote!(#name(#(#args),*))
    };
    let result = if fallible {
        quote!(#call.map_err(|error| error.to_string()))
    } else {
        quote!(Ok(#call))
    };
    let mut attrs = Vec::new();
    add_derives(&mut attrs, &["Deserialize", "TS"])?;
    let request_definition = if fields.is_empty() {
        quote!()
    } else {
        quote!(#(#attrs)* #[serde(deny_unknown_fields)] struct #request { #(#fields),* })
    };
    let request = if fields.is_empty() {
        quote!(())
    } else {
        quote!(#request)
    };
    let constructor = if function.sig.asyncness.is_some() {
        quote!(::solid_gpui::native::CommandDefinition::asynchronous::<#request, #output, _, _>(#js_name, |__request: #request| async move { #result }))
    } else {
        quote!(::solid_gpui::native::CommandDefinition::sync::<#request, #output>(#js_name, |__request: #request| { #result }))
    };
    Ok((
        quote! {
            #(#helpers)*
            #request_definition
            fn #helper() -> ::solid_gpui::native::CommandDefinition { #constructor }
        },
        quote!(#helper()),
    ))
}

fn expand_module(options: Tokens, mut module: ItemMod) -> syn::Result<Tokens> {
    let contract = quote!(#options #module).to_string();
    let mut namespace: Option<LitStr> = None;
    let parser = syn::meta::parser(|meta| {
        if !meta.path.is_ident("name") {
            return Err(meta.error("unknown native_module option; expected name"));
        }
        if namespace.is_some() {
            return Err(meta.error("duplicate native_module name"));
        }
        namespace = Some(meta.value()?.parse()?);
        Ok(())
    });
    syn::parse::Parser::parse2(parser, options)?;
    let Some((_, items)) = &mut module.content else {
        return Err(syn::Error::new_spanned(
            module,
            "native_module requires an inline module",
        ));
    };
    let mut components = Vec::new();
    let mut commands = Vec::new();
    let mut generated = Vec::new();
    for item in items.iter_mut() {
        match item {
            Item::Fn(function) => {
                let component = function.attrs.iter().any(|attr| is_attr(attr, "component"));
                let command = function.attrs.iter().any(|attr| is_attr(attr, "command"));
                if component && command {
                    return Err(syn::Error::new_spanned(
                        function,
                        "an export cannot be both component and command",
                    ));
                }
                let conditions: Vec<_> = function
                    .attrs
                    .iter()
                    .filter(|attr| is_attr(attr, "cfg"))
                    .cloned()
                    .collect();
                if component {
                    for attr in &mut function.attrs {
                        if is_attr(attr, "component") {
                            qualify_component_attribute(attr)?;
                        }
                    }
                    let definition = format_ident!("__native_component_{}", function.sig.ident);
                    components.push(quote!(#(#conditions)* #definition()));
                } else if command {
                    for attr in function
                        .attrs
                        .iter()
                        .filter(|attr| is_attr(attr, "command"))
                    {
                        if !matches!(attr.meta, Meta::Path(_)) {
                            return Err(syn::Error::new_spanned(
                                attr,
                                "command accepts no options",
                            ));
                        }
                    }
                    function.attrs.retain(|attr| !is_attr(attr, "command"));
                    let (helper, constructor) = expand_command(function)?;
                    let mut helper: syn::File = syn::parse2(helper)?;
                    for item in &mut helper.items {
                        match item {
                            Item::Fn(item) => item.attrs.extend(conditions.iter().cloned()),
                            Item::Struct(item) => item.attrs.extend(conditions.iter().cloned()),
                            _ => unreachable!(),
                        }
                    }
                    generated.push(quote!(#helper));
                    commands.push(quote!(#(#conditions)* #constructor));
                }
            }
            Item::Impl(implementation)
                if implementation
                    .attrs
                    .iter()
                    .any(|attr| is_attr(attr, "component")) =>
            {
                let name = view_name(implementation)?;
                let conditions: Vec<_> = implementation
                    .attrs
                    .iter()
                    .filter(|attr| is_attr(attr, "cfg"))
                    .cloned()
                    .collect();
                for attr in &mut implementation.attrs {
                    if is_attr(attr, "component") {
                        qualify_component_attribute(attr)?;
                    }
                }
                let definition = format_ident!("__native_component_{}", name);
                components.push(quote!(#(#conditions)* #definition()));
            }
            _ => {}
        }
    }
    if items
        .iter()
        .any(|item| matches!(item, Item::Fn(function) if function.sig.ident == "native_module"))
    {
        return Err(syn::Error::new(
            module.ident.span(),
            "native_module reserves the function name native_module",
        ));
    }
    let namespace = namespace.map_or_else(
        || quote!(concat!(env!("CARGO_PKG_NAME"), "::", module_path!())),
        |name| quote!(#name),
    );
    let registration = quote! {
        #(#generated)*
        pub fn native_module() -> ::solid_gpui::native::ModuleDefinition {
            ::solid_gpui::native::ModuleDefinition::new(#namespace, vec![#(#components),*], vec![#(#commands),*]).with_contract(#contract)
        }
    };
    let generated: syn::File = syn::parse2(registration)?;
    items.extend(generated.items);
    Ok(quote!(#module))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_contract_rejects_serde_divergence() {
        for attribute in [
            quote!(#[serde(flatten)]),
            quote!(#[serde(with = "adapter")]),
            quote!(#[serde(rename(serialize = "out", deserialize = "in"))]),
            quote!(#[ts(type = "any")]),
        ] {
            let item = syn::parse2(quote!(struct Input { #attribute value: String })).unwrap();
            assert!(expand_native_type(item).is_err());
        }
        let item = parse_quote!(
            #[derive(Clone)]
            #[serde(rename_all = "camelCase")]
            struct Input {
                #[serde(default)]
                value: Option<String>,
            }
        );
        assert!(expand_native_type(item).is_ok());
    }

    #[test]
    fn components_separate_props_events_and_context() {
        let function = parse_quote!(
            fn greeting(
                name: String,
                #[prop(default = true)] disabled: bool,
                on_press: Event<()>,
                cx: &mut ElementContext,
            ) -> impl IntoElement {
                todo!()
            }
        );
        let output = expand_component(function, true, false, None)
            .unwrap()
            .to_string();
        assert!(output.contains("__native_default_greeting_disabled"));
        assert!(output.contains("event :: < () > (\"press\")"));
        let file: syn::File = syn::parse_str(&output).unwrap();
        let props = file
            .items
            .iter()
            .find_map(|item| {
                if let Item::Struct(item) = item {
                    Some(item)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(props.fields.len(), 2);
        for function in [
            parse_quote!(
                fn bad(x: &str) {}
            ),
            parse_quote!(
                fn bad(#[prop(unknown)] x: bool) {}
            ),
            parse_quote!(
                fn bad<T>(x: T) {}
            ),
        ] {
            assert!(expand_component(function, true, false, None).is_err());
        }
    }

    #[test]
    fn module_collects_commands_and_stateful_views() {
        let module = parse_quote!(
            mod app {
                #[command]
                async fn load(path: String) -> Result<String, std::io::Error> {
                    todo!()
                }
                #[component]
                fn button(cx: &mut ElementContext) -> impl IntoElement {
                    todo!()
                }
                #[component]
                impl NativeView for Editor {
                    type Props = EditorProps;
                }
            }
        );
        let result = expand_module(quote!(name = "app"), module)
            .unwrap()
            .to_string();
        assert!(result.contains("asynchronous ::"));
        assert!(result.contains("__native_component_Editor ()"));
        let file = syn::parse2::<syn::File>(result.parse().unwrap()).unwrap();
        let Item::Mod(module) = &file.items[0] else {
            panic!("expected module");
        };
        assert!(module.content.as_ref().unwrap().1.iter().all(|item| {
            !matches!(item, Item::Fn(function) if function.attrs.iter().any(|attr| is_attr(attr, "command")))
        }));
    }
}
