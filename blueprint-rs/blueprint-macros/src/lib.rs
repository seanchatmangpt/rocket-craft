//! Procedural macros for writing UE4 Blueprint graphs with a Rust-like DSL.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use blueprint_macros::blueprint;
//!
//! let t3d = blueprint! {
//!     name: "MyActor",
//!     parent: "Actor",
//!
//!     var health: i32 = 100,
//!     var speed: f32 = 600.0,
//!     var is_alive: bool = true,
//!
//!     on begin_play {
//!         print("Hello from Blueprint-RS!");
//!         call some_function("arg1");
//!         set health = 50;
//!         branch is_alive {
//!             true => print("alive!"),
//!             false => print("dead!"),
//!         }
//!     }
//!
//!     on tick(delta: float) {
//!         call update_movement("DeltaSeconds");
//!     }
//!
//!     on custom("MyEvent") {
//!         set speed = 300.0;
//!     }
//! };
//! ```
//!
//! The macro expands to a `blueprint_core::BlueprintBuilder` call chain and
//! returns the T3D string produced by `.to_t3d()`.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitFloat, LitInt, LitStr, Result, Token,
};

// ---------------------------------------------------------------------------
// AST types
// ---------------------------------------------------------------------------

/// Top-level blueprint definition parsed from the macro input.
struct BlueprintDef {
    name: String,
    parent: String,
    variables: Vec<VarDef>,
    events: Vec<EventDef>,
}

/// A Blueprint variable: `var name: type = default`.
struct VarDef {
    name: String,
    ty: VarType,
    default: Option<String>,
}

/// Supported variable types.
enum VarType {
    Int,
    Float,
    Bool,
    String,
    Name,
}

/// An event handler: `on begin_play { … }` etc.
struct EventDef {
    kind: EventKind,
    body: Vec<Statement>,
}

/// Which event this graph handles.
enum EventKind {
    BeginPlay,
    EndPlay,
    Tick,
    Custom(String),
}

/// A single statement inside an event body.
enum Statement {
    Print(String),
    CallFunction { func: String, args: Vec<String> },
    SetVar { name: String, value: String },
    GetVar { name: String },
    Branch { condition: String },
    ForLoop { var: String, start: i32, end: i32 },
}

// ---------------------------------------------------------------------------
// Parse helpers
// ---------------------------------------------------------------------------

/// Try to parse an optional trailing comma and return true if one was consumed.
fn eat_comma(input: ParseStream) -> bool {
    input.parse::<Token![,]>().is_ok()
}

/// Parse a `"string"` or bare identifier as a plain `String`.
fn parse_string_value(input: ParseStream) -> Result<String> {
    if input.peek(LitStr) {
        Ok(input.parse::<LitStr>()?.value())
    } else {
        Ok(input.parse::<Ident>()?.to_string())
    }
}

// ---------------------------------------------------------------------------
// Parse implementations
// ---------------------------------------------------------------------------

impl Parse for VarType {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "i32" | "i64" | "u32" | "u64" | "int" | "integer" => Ok(VarType::Int),
            "f32" | "f64" | "float" => Ok(VarType::Float),
            "bool" | "boolean" => Ok(VarType::Bool),
            "String" | "string" | "str" => Ok(VarType::String),
            "Name" | "name" => Ok(VarType::Name),
            other => Err(syn::Error::new(
                ident.span(),
                format!("unknown Blueprint variable type `{other}`; expected i32, f32, bool, String, or Name"),
            )),
        }
    }
}

impl Parse for VarDef {
    /// Parses: `var name: Type = default`
    fn parse(input: ParseStream) -> Result<Self> {
        // `var` keyword (parsed as Ident)
        let kw: Ident = input.parse()?;
        if kw != "var" {
            return Err(syn::Error::new(kw.span(), "expected `var`"));
        }

        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: VarType = input.parse()?;

        let default = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            // Accept a string literal, integer literal, float literal, or ident (true/false)
            let val = if input.peek(LitStr) {
                input.parse::<LitStr>()?.value()
            } else if input.peek(LitFloat) {
                input.parse::<LitFloat>()?.to_string()
            } else if input.peek(LitInt) {
                input.parse::<LitInt>()?.to_string()
            } else {
                // bool literal or identifier
                let id: Ident = input.parse()?;
                id.to_string()
            };
            Some(val)
        } else {
            None
        };

        Ok(VarDef {
            name: name.to_string(),
            ty,
            default,
        })
    }
}

impl Parse for Statement {
    /// Parses one statement inside an event body.
    fn parse(input: ParseStream) -> Result<Self> {
        let kw: Ident = input.parse()?;
        match kw.to_string().as_str() {
            // print("msg") or print_string("msg")
            "print" | "print_string" => {
                let content;
                syn::parenthesized!(content in input);
                let msg = parse_string_value(&content)?;
                eat_comma(input);
                Ok(Statement::Print(msg))
            }

            // call func(arg1, arg2, …)
            "call" => {
                let func_ident: Ident = input.parse()?;
                let func = func_ident.to_string();
                let mut args = Vec::new();
                if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);
                    while !content.is_empty() {
                        args.push(parse_string_value(&content)?);
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                eat_comma(input);
                Ok(Statement::CallFunction { func, args })
            }

            // set var_name = value
            "set" => {
                let name: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value = if input.peek(LitStr) {
                    input.parse::<LitStr>()?.value()
                } else if input.peek(LitFloat) {
                    input.parse::<LitFloat>()?.to_string()
                } else if input.peek(LitInt) {
                    input.parse::<LitInt>()?.to_string()
                } else {
                    let id: Ident = input.parse()?;
                    id.to_string()
                };
                eat_comma(input);
                Ok(Statement::SetVar {
                    name: name.to_string(),
                    value,
                })
            }

            // get var_name
            "get" => {
                let name: Ident = input.parse()?;
                eat_comma(input);
                Ok(Statement::GetVar {
                    name: name.to_string(),
                })
            }

            // branch condition { true => stmt, false => stmt }
            "branch" => {
                let condition = parse_string_value(input)?;
                // Parse optional `{ true => …, false => … }` block (we record the condition only
                // for now; branch arms are not yet recursively parsed but we consume them).
                if input.peek(syn::token::Brace) {
                    let arm_content;
                    braced!(arm_content in input);
                    // Consume arms without recursive statement parsing (keeps grammar simple)
                    while !arm_content.is_empty() {
                        // true / false
                        let _arm_label: Ident = arm_content.parse()?;
                        arm_content.parse::<Token![=>]>()?;
                        // Consume a single statement or a block
                        if arm_content.peek(syn::token::Brace) {
                            let inner;
                            braced!(inner in arm_content);
                            // drain inner
                            while !inner.is_empty() {
                                inner.parse::<proc_macro2::TokenTree>()?;
                            }
                        } else {
                            // single-expression arm: consume up to next comma or closing brace
                            while !arm_content.is_empty()
                                && !arm_content.peek(Token![,])
                                && !arm_content.peek(syn::token::Brace)
                            {
                                arm_content.parse::<proc_macro2::TokenTree>()?;
                            }
                        }
                        // optional trailing comma between arms
                        if arm_content.peek(Token![,]) {
                            arm_content.parse::<Token![,]>()?;
                        }
                    }
                }
                eat_comma(input);
                Ok(Statement::Branch { condition })
            }

            // for i in start..end { … }
            "for" => {
                let var_ident: Ident = input.parse()?;
                let var = var_ident.to_string();
                // `in`
                let _in_kw: Ident = input.parse()?;
                let start: LitInt = input.parse()?;
                input.parse::<Token![..]>()?;
                let end: LitInt = input.parse()?;

                // consume the body block
                if input.peek(syn::token::Brace) {
                    let body;
                    braced!(body in input);
                    while !body.is_empty() {
                        body.parse::<proc_macro2::TokenTree>()?;
                    }
                }
                eat_comma(input);
                Ok(Statement::ForLoop {
                    var,
                    start: start.base10_parse()?,
                    end: end.base10_parse()?,
                })
            }

            other => Err(syn::Error::new(
                kw.span(),
                format!(
                    "unknown statement `{other}`; expected print, call, set, get, branch, or for"
                ),
            )),
        }
    }
}

impl Parse for EventDef {
    /// Parses: `on begin_play { … }` | `on tick(delta: float) { … }` | `on custom("Name") { … }`
    fn parse(input: ParseStream) -> Result<Self> {
        // `on` keyword
        let on_kw: Ident = input.parse()?;
        if on_kw != "on" {
            return Err(syn::Error::new(on_kw.span(), "expected `on`"));
        }

        let event_ident: Ident = input.parse()?;
        let kind = match event_ident.to_string().as_str() {
            "begin_play" | "BeginPlay" => {
                // optional empty param list: `begin_play()`
                if input.peek(syn::token::Paren) {
                    let _discard;
                    syn::parenthesized!(_discard in input);
                }
                EventKind::BeginPlay
            }
            "end_play" | "EndPlay" => {
                if input.peek(syn::token::Paren) {
                    let _discard;
                    syn::parenthesized!(_discard in input);
                }
                EventKind::EndPlay
            }
            "tick" | "Tick" => {
                // optional param list `(delta: float)` — we just consume it
                if input.peek(syn::token::Paren) {
                    let params;
                    syn::parenthesized!(params in input);
                    while !params.is_empty() {
                        params.parse::<proc_macro2::TokenTree>()?;
                    }
                }
                EventKind::Tick
            }
            "custom" | "Custom" => {
                // `custom("EventName")`
                let name_content;
                syn::parenthesized!(name_content in input);
                let name: LitStr = name_content.parse()?;
                EventKind::Custom(name.value())
            }
            other => {
                return Err(syn::Error::new(
                    event_ident.span(),
                    format!("unknown event `{other}`; expected begin_play, end_play, tick, or custom(\"…\")"),
                ))
            }
        };

        // Parse the body block `{ stmt* }`
        let body_content;
        braced!(body_content in input);
        let mut body = Vec::new();
        while !body_content.is_empty() {
            // Skip standalone semicolons
            if body_content.peek(Token![;]) {
                body_content.parse::<Token![;]>()?;
                continue;
            }
            body.push(body_content.parse::<Statement>()?);
        }

        Ok(EventDef { kind, body })
    }
}

impl Parse for BlueprintDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = String::from("MyBlueprint");
        let mut parent = String::from("Actor");
        let mut variables = Vec::new();
        let mut events = Vec::new();

        while !input.is_empty() {
            // Skip stray commas / semicolons between top-level items.
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                continue;
            }
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                continue;
            }

            let lookahead = input.lookahead1();
            if !lookahead.peek(Ident) {
                return Err(lookahead.error());
            }

            // Peek at the identifier to decide what to parse.
            let kw: Ident = input.fork().parse()?;
            match kw.to_string().as_str() {
                "name" => {
                    input.parse::<Ident>()?; // consume `name`
                    input.parse::<Token![:]>()?;
                    name = input.parse::<LitStr>()?.value();
                    eat_comma(input);
                }
                "parent" => {
                    input.parse::<Ident>()?;
                    input.parse::<Token![:]>()?;
                    parent = input.parse::<LitStr>()?.value();
                    eat_comma(input);
                }
                "var" => {
                    variables.push(input.parse::<VarDef>()?);
                    eat_comma(input);
                }
                "on" => {
                    events.push(input.parse::<EventDef>()?);
                }
                other => {
                    return Err(syn::Error::new(
                        kw.span(),
                        format!("unexpected keyword `{other}` at the top level of blueprint!"),
                    ))
                }
            }
        }

        Ok(BlueprintDef {
            name,
            parent,
            variables,
            events,
        })
    }
}

// ---------------------------------------------------------------------------
// Code generation helpers
// ---------------------------------------------------------------------------

fn var_type_tokens(ty: &VarType) -> TokenStream2 {
    match ty {
        VarType::Int => quote! { ::blueprint_core::VarType::Int },
        VarType::Float => quote! { ::blueprint_core::VarType::Float },
        VarType::Bool => quote! { ::blueprint_core::VarType::Bool },
        VarType::String => quote! { ::blueprint_core::VarType::String },
        VarType::Name => quote! { ::blueprint_core::VarType::Name },
    }
}

fn statement_tokens(stmt: &Statement) -> TokenStream2 {
    match stmt {
        Statement::Print(msg) => {
            quote! { __ev.print(#msg); }
        }
        Statement::CallFunction { func, args } => {
            let args_ts: Vec<TokenStream2> = args
                .iter()
                .map(|a| quote! { ::std::string::String::from(#a) })
                .collect();
            quote! {
                __ev.call(#func, vec![#(#args_ts),*]);
            }
        }
        Statement::SetVar { name, value } => {
            quote! { __ev.set_var(#name, #value); }
        }
        Statement::GetVar { name } => {
            quote! { __ev.get_var(#name); }
        }
        Statement::Branch { condition } => {
            quote! { __ev.branch(#condition); }
        }
        Statement::ForLoop { var, start, end } => {
            quote! { __ev.for_loop(#var, #start, #end); }
        }
    }
}

fn event_chain_tokens(event: &EventDef) -> TokenStream2 {
    let body_calls: Vec<TokenStream2> = event.body.iter().map(statement_tokens).collect();

    let body_closure = quote! {
        |__ev: &mut ::blueprint_core::EventBodyBuilder| {
            #(#body_calls)*
        }
    };

    match &event.kind {
        EventKind::BeginPlay => quote! {
            .begin_play(#body_closure)
        },
        EventKind::EndPlay => quote! {
            .end_play(#body_closure)
        },
        EventKind::Tick => quote! {
            .tick(#body_closure)
        },
        EventKind::Custom(name) => quote! {
            .custom_event(#name, #body_closure)
        },
    }
}

// ---------------------------------------------------------------------------
// blueprint! macro
// ---------------------------------------------------------------------------

/// DSL macro for building a UE4 Blueprint and returning its T3D string.
///
/// See the [crate-level documentation](index.html) for full syntax.
#[proc_macro]
pub fn blueprint(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as BlueprintDef);

    let bp_name = &def.name;
    let bp_parent = &def.parent;

    // Variable builder calls
    let var_calls: Vec<TokenStream2> = def
        .variables
        .iter()
        .map(|v| {
            let var_name = &v.name;
            let var_ty = var_type_tokens(&v.ty);
            match &v.default {
                Some(d) => quote! {
                    .variable(#var_name, #var_ty, ::core::option::Option::Some(::std::string::String::from(#d)))
                },
                None => quote! {
                    .variable(#var_name, #var_ty, ::core::option::Option::None)
                },
            }
        })
        .collect();

    // Event builder calls
    let event_calls: Vec<TokenStream2> = def.events.iter().map(event_chain_tokens).collect();

    let expanded = quote! {
        {
            ::blueprint_core::BlueprintBuilder::new(#bp_name, #bp_parent)
                #(#var_calls)*
                #(#event_calls)*
                .to_t3d()
        }
    };

    expanded.into()
}

// ---------------------------------------------------------------------------
// bp_node! macro
// ---------------------------------------------------------------------------

/// A single T3D-style node with class, name, and optional key=value properties.
///
/// ```rust,ignore
/// let node = bp_node! {
///     class: "K2Node_Event",
///     name: "Event_0",
///     props: {
///         EventReference: "ReceiveBeginPlay",
///         bOverrideFunction: "True",
///     }
/// };
/// ```
#[proc_macro]
pub fn bp_node(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as BpNodeDef);

    let class = &parsed.class;
    let node_name = &parsed.name;
    let props: Vec<TokenStream2> = parsed
        .props
        .iter()
        .map(|(k, v)| {
            quote! { ::std::format!("   {}=\"{}\"\n", #k, #v) }
        })
        .collect();

    let expanded = quote! {
        {
            let mut __out = ::std::string::String::new();
            __out.push_str(&::std::format!(
                "Begin Object Class=/Script/BlueprintGraph.{}\n",
                #class
            ));
            __out.push_str(&::std::format!("   Name=\"{}\"\n", #node_name));
            #(
                __out.push_str(&#props);
            )*
            __out.push_str("End Object\n");
            __out
        }
    };

    expanded.into()
}

// ---------------------------------------------------------------------------
// bp_node! parser
// ---------------------------------------------------------------------------

struct BpNodeDef {
    class: String,
    name: String,
    props: Vec<(String, String)>,
}

impl Parse for BpNodeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut class = String::from("K2Node_Event");
        let mut name = String::from("Node_0");
        let mut props = Vec::new();

        while !input.is_empty() {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                continue;
            }

            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "class" => {
                    class = input.parse::<LitStr>()?.value();
                    eat_comma(input);
                }
                "name" => {
                    name = input.parse::<LitStr>()?.value();
                    eat_comma(input);
                }
                "props" => {
                    let props_content;
                    braced!(props_content in input);
                    while !props_content.is_empty() {
                        if props_content.peek(Token![,]) {
                            props_content.parse::<Token![,]>()?;
                            continue;
                        }
                        let prop_key: Ident = props_content.parse()?;
                        props_content.parse::<Token![:]>()?;
                        let prop_val = parse_string_value(&props_content)?;
                        props.push((prop_key.to_string(), prop_val));
                        if props_content.peek(Token![,]) {
                            props_content.parse::<Token![,]>()?;
                        }
                    }
                    eat_comma(input);
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unexpected key `{other}` in bp_node!; expected class, name, or props"
                        ),
                    ))
                }
            }
        }

        Ok(BpNodeDef { class, name, props })
    }
}
