//! High-level [`BlueprintBuilder`] — fluent builder for constructing Blueprint
//! graphs.
//!
//! Wraps a [`Blueprint`] and provides ergonomic methods for adding nodes and
//! wiring them together.  All nodes are added to the EventGraph by default.
//!
//! The builder exposes **two styles**:
//!
//! * **Mutable (`&mut self`)** — for imperative use where you build a graph
//!   step by step, collecting [`NodeHandle`]s so you can wire pins afterwards.
//! * **Consuming (`self`)** — used by the `blueprint_macros` proc-macro crate,
//!   where the DSL expands to a method-chain that ends with `.to_t3d()`.

use crate::ast::{BpNode, Blueprint, Pin};
use crate::nodes;
use crate::serializer::{JsonSerializer, T3dSerializer};
use crate::types::{NodePos, PinType};

// ---------------------------------------------------------------------------
// VarType — simplified variable-type enum used by the macro DSL
// ---------------------------------------------------------------------------

/// Simplified variable type used by the `blueprint!` macro DSL.
///
/// Maps to the corresponding [`PinType`] constructors.
#[derive(Debug, Clone, PartialEq)]
pub enum VarType {
    Int,
    Float,
    Bool,
    String,
    Name,
}

impl VarType {
    /// UE4 T3D property-type name.
    pub fn as_t3d_type(&self) -> &'static str {
        match self {
            VarType::Int => "IntProperty",
            VarType::Float => "FloatProperty",
            VarType::Bool => "BoolProperty",
            VarType::String => "StrProperty",
            VarType::Name => "NameProperty",
        }
    }

    /// Convert to a low-level [`PinType`].
    pub fn to_pin_type(&self) -> PinType {
        match self {
            VarType::Int => PinType::int(),
            VarType::Float => PinType::float(),
            VarType::Bool => PinType::bool(),
            VarType::String => PinType::string(),
            VarType::Name => PinType::name(),
        }
    }
}

// ---------------------------------------------------------------------------
// NodeHandle
// ---------------------------------------------------------------------------

/// A lightweight handle returned by builder node methods so callers can wire
/// nodes without holding references into the builder.
#[derive(Debug, Clone)]
pub struct NodeHandle {
    pub name: std::string::String,
}

impl NodeHandle {
    pub fn new(name: impl Into<std::string::String>) -> Self {
        Self { name: name.into() }
    }
}

// ---------------------------------------------------------------------------
// EventBodyBuilder — used in macro closures
// ---------------------------------------------------------------------------

/// A single statement / Blueprint node inside an event body.
#[derive(Debug, Clone)]
pub enum Statement {
    /// `print("msg")` → K2Node_CallFunction → PrintString
    Print(std::string::String),
    /// `call func(arg, …)` → K2Node_CallFunction
    CallFunction {
        func: std::string::String,
        args: Vec<std::string::String>,
    },
    /// `set var = value` → K2Node_VariableSet
    SetVar {
        name: std::string::String,
        value: std::string::String,
    },
    /// `get var` → K2Node_VariableGet
    GetVar { name: std::string::String },
    /// `branch condition { … }` → K2Node_IfThenElse
    Branch { condition: std::string::String },
    /// `for i in start..end { … }` → K2Node_MacroInstance (ForLoop)
    ForLoop {
        var: std::string::String,
        start: i32,
        end: i32,
    },
}

/// Accumulates [`Statement`]s inside an event-body closure passed to the
/// consuming builder methods (`begin_play`, `tick`, etc.).
pub struct EventBodyBuilder {
    pub statements: Vec<Statement>,
}

impl EventBodyBuilder {
    pub fn new() -> Self {
        EventBodyBuilder {
            statements: Vec::new(),
        }
    }

    /// Append a PrintString node.
    pub fn print(&mut self, msg: impl Into<std::string::String>) -> &mut Self {
        self.statements.push(Statement::Print(msg.into()));
        self
    }

    /// Append a CallFunction node.
    pub fn call(
        &mut self,
        func: impl Into<std::string::String>,
        args: Vec<std::string::String>,
    ) -> &mut Self {
        self.statements.push(Statement::CallFunction {
            func: func.into(),
            args,
        });
        self
    }

    /// Append a VariableSet node.
    pub fn set_var(
        &mut self,
        name: impl Into<std::string::String>,
        value: impl Into<std::string::String>,
    ) -> &mut Self {
        self.statements.push(Statement::SetVar {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    /// Append a VariableGet node.
    pub fn get_var(&mut self, name: impl Into<std::string::String>) -> &mut Self {
        self.statements
            .push(Statement::GetVar { name: name.into() });
        self
    }

    /// Append a Branch (IfThenElse) node.
    pub fn branch(&mut self, condition: impl Into<std::string::String>) -> &mut Self {
        self.statements.push(Statement::Branch {
            condition: condition.into(),
        });
        self
    }

    /// Append a ForLoop node.
    pub fn for_loop(
        &mut self,
        var: impl Into<std::string::String>,
        start: i32,
        end: i32,
    ) -> &mut Self {
        self.statements.push(Statement::ForLoop {
            var: var.into(),
            start,
            end,
        });
        self
    }
}

impl Default for EventBodyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// BlueprintBuilder
// ---------------------------------------------------------------------------

/// Fluent builder for constructing Blueprint graphs that serialises to UE4 T3D.
///
/// # Example — mutable (imperative) style
/// ```rust,ignore
/// let mut builder = BlueprintBuilder::new("MyActor", "Actor");
/// let bp_node = builder.begin_play_node();
/// let ps_node = builder.print_string("Hello!");
/// builder.exec_connect(&bp_node, &ps_node);
/// let t3d = builder.to_t3d();
/// ```
///
/// # Example — consuming (macro) style
/// ```rust
/// # use blueprint_core::{BlueprintBuilder, VarType};
/// let t3d = BlueprintBuilder::new("MyActor", "Actor")
///     .variable("Health", VarType::Int, Some("100".into()))
///     .begin_play(|ev| {
///         ev.print("Hello from Blueprint-RS!");
///     })
///     .to_t3d();
/// assert!(t3d.contains("PrintString"));
/// ```
pub struct BlueprintBuilder {
    blueprint: Blueprint,
    /// Auto-incrementing counter for unique node names
    node_counter: u32,
    /// X offset for auto-layout
    next_x: i32,
}

impl BlueprintBuilder {
    /// Create a new builder.
    pub fn new(
        name: impl Into<std::string::String>,
        parent_class: impl Into<std::string::String>,
    ) -> Self {
        Self {
            blueprint: Blueprint::new(name, parent_class),
            node_counter: 0,
            next_x: 0,
        }
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    fn unique_name(&mut self, base: &str) -> std::string::String {
        self.node_counter += 1;
        format!("{}_{}", base, self.node_counter)
    }

    fn next_pos(&mut self) -> NodePos {
        let x = self.next_x;
        self.next_x += 250;
        NodePos::new(x, 0)
    }

    fn push_node(&mut self, node: BpNode) -> NodeHandle {
        let name = node.name.clone();
        self.blueprint.event_graph().nodes.push(node);
        NodeHandle::new(name)
    }

    // ------------------------------------------------------------------
    // Mutable-style helpers for variable management
    // ------------------------------------------------------------------

    /// Add a Blueprint variable (mutable style).
    pub fn add_variable_mut(&mut self, name: impl Into<std::string::String>, ty: VarType, default: Option<std::string::String>) -> &mut Self {
        use crate::ast::BpVariable;
        let mut var = BpVariable::new(name.into(), ty.to_pin_type());
        if let Some(d) = default {
            var = var.with_default(d);
        }
        self.blueprint.add_variable(var);
        self
    }

    // ------------------------------------------------------------------
    // Consuming-style wrappers (used by the blueprint! macro)
    // ------------------------------------------------------------------

    /// Add a variable (consuming style — used by `blueprint!` macro).
    pub fn variable(
        mut self,
        name: impl Into<std::string::String>,
        ty: VarType,
        default: Option<std::string::String>,
    ) -> Self {
        use crate::ast::BpVariable;
        let mut var = BpVariable::new(name.into(), ty.to_pin_type());
        if let Some(d) = default {
            var = var.with_default(d);
        }
        self.blueprint.add_variable(var);
        self
    }

    /// Add a BeginPlay event and body statements (consuming style).
    pub fn begin_play<F>(mut self, build: F) -> Self
    where
        F: FnOnce(&mut EventBodyBuilder),
    {
        let mut eb = EventBodyBuilder::new();
        build(&mut eb);
        let name = self.unique_name("BeginPlay");
        let pos = self.next_pos();
        let node = nodes::begin_play(&name).at(pos.x, pos.y);
        self.push_node(node);
        self.push_statements(eb.statements);
        self
    }

    /// Add an EndPlay event and body statements (consuming style).
    pub fn end_play<F>(mut self, build: F) -> Self
    where
        F: FnOnce(&mut EventBodyBuilder),
    {
        let mut eb = EventBodyBuilder::new();
        build(&mut eb);
        let name = self.unique_name("EndPlay");
        let pos = self.next_pos();
        let node = nodes::end_play(&name).at(pos.x, pos.y);
        self.push_node(node);
        self.push_statements(eb.statements);
        self
    }

    /// Add a Tick event and body statements (consuming style).
    pub fn tick<F>(mut self, build: F) -> Self
    where
        F: FnOnce(&mut EventBodyBuilder),
    {
        let mut eb = EventBodyBuilder::new();
        build(&mut eb);
        let name = self.unique_name("Tick");
        let pos = self.next_pos();
        let node = nodes::tick(&name).at(pos.x, pos.y);
        self.push_node(node);
        self.push_statements(eb.statements);
        self
    }

    /// Add a custom event and body statements (consuming style).
    pub fn custom_event<F>(mut self, event_name: impl Into<std::string::String>, build: F) -> Self
    where
        F: FnOnce(&mut EventBodyBuilder),
    {
        let event_name = event_name.into();
        let mut eb = EventBodyBuilder::new();
        build(&mut eb);
        let node_name = self.unique_name(&format!("CustomEvent_{event_name}"));
        let pos = self.next_pos();
        let node = nodes::custom_event(&node_name, &event_name).at(pos.x, pos.y);
        self.push_node(node);
        self.push_statements(eb.statements);
        self
    }

    /// Expand a list of statements into nodes in the event graph.
    fn push_statements(&mut self, stmts: Vec<Statement>) {
        for stmt in stmts {
            let pos = self.next_pos();
            let idx = self.node_counter;
            self.node_counter += 1;
            let node = match stmt {
                Statement::Print(msg) => {
                    let name = format!("PrintString_{idx}");
                    let mut in_str = Pin::data_input("InString", PinType::string());
                    in_str.default_value = Some(msg);
                    BpNode::new(
                        "/Script/BlueprintGraph.K2Node_CallFunction",
                        &name,
                    )
                    .at(pos.x, pos.y)
                    .with_property(
                        "FunctionReference",
                        "(MemberParent=Class'/Script/Engine.KismetSystemLibrary',MemberName=\"PrintString\")",
                    )
                    .with_pin(Pin::exec_input("execute"))
                    .with_pin(Pin::exec_output("then"))
                    .with_pin(in_str)
                }
                Statement::CallFunction { func, args } => {
                    let name = format!("CallFunction_{idx}");
                    let mut node = BpNode::new(
                        "/Script/BlueprintGraph.K2Node_CallFunction",
                        &name,
                    )
                    .at(pos.x, pos.y)
                    .with_property(
                        "FunctionReference",
                        &format!("(MemberName=\"{func}\")"),
                    )
                    .with_pin(Pin::exec_input("execute"))
                    .with_pin(Pin::exec_output("then"));
                    for (i, arg) in args.iter().enumerate() {
                        let mut p = Pin::data_input(&format!("Param{i}"), PinType::wildcard());
                        p.default_value = Some(arg.clone());
                        node = node.with_pin(p);
                    }
                    node
                }
                Statement::SetVar { name: var_name, value } => {
                    let name = format!("VariableSet_{idx}");
                    let mut val_pin = Pin::data_input("NewValue", PinType::wildcard());
                    val_pin.default_value = Some(value);
                    BpNode::new(
                        "/Script/BlueprintGraph.K2Node_VariableSet",
                        &name,
                    )
                    .at(pos.x, pos.y)
                    .with_property("VariableName", &format!("\"{var_name}\""))
                    .with_pin(Pin::exec_input("execute"))
                    .with_pin(Pin::exec_output("then"))
                    .with_pin(val_pin)
                }
                Statement::GetVar { name: var_name } => {
                    let name = format!("VariableGet_{idx}");
                    BpNode::new(
                        "/Script/BlueprintGraph.K2Node_VariableGet",
                        &name,
                    )
                    .at(pos.x, pos.y)
                    .with_property("VariableName", &format!("\"{var_name}\""))
                    .with_pin(Pin::data_output("Value", PinType::wildcard()))
                }
                Statement::Branch { condition } => {
                    let name = format!("Branch_{idx}");
                    let mut cond_pin = Pin::data_input("Condition", PinType::bool());
                    cond_pin.default_value = Some(condition);
                    nodes::branch_node(&name)
                        .at(pos.x, pos.y)
                }
                Statement::ForLoop { var, start, end } => {
                    let name = format!("ForLoop_{idx}");
                    BpNode::new(
                        "/Script/BlueprintGraph.K2Node_MacroInstance",
                        &name,
                    )
                    .at(pos.x, pos.y)
                    .with_property("MacroGraphReference", "(MacroName=\"ForLoop\")")
                    .with_property("LoopVar", &format!("\"{var}\""))
                    .with_property("StartIndex", &start.to_string())
                    .with_property("LastIndex", &(end - 1).to_string())
                    .with_pin(Pin::exec_input("execute"))
                    .with_pin(Pin::exec_output("LoopBody"))
                    .with_pin(Pin::exec_output("Completed"))
                }
            };
            self.blueprint.event_graph().nodes.push(node);
        }
    }

    // ------------------------------------------------------------------
    // Mutable-style event node additions (imperative use)
    // ------------------------------------------------------------------

    /// Add a BeginPlay event node (mutable style, returns a handle).
    pub fn begin_play_node(&mut self) -> NodeHandle {
        let name = self.unique_name("BeginPlay");
        let pos = self.next_pos();
        let node = nodes::begin_play(&name).at(pos.x, pos.y);
        self.push_node(node)
    }

    /// Add an EndPlay event node (mutable style).
    pub fn end_play_node(&mut self) -> NodeHandle {
        let name = self.unique_name("EndPlay");
        let pos = self.next_pos();
        let node = nodes::end_play(&name).at(pos.x, pos.y);
        self.push_node(node)
    }

    /// Add a Tick event node (mutable style).
    pub fn tick_node(&mut self) -> NodeHandle {
        let name = self.unique_name("Tick");
        let pos = self.next_pos();
        let node = nodes::tick(&name).at(pos.x, pos.y);
        self.push_node(node)
    }

    /// Add a custom event node (mutable style).
    pub fn custom_event_node(&mut self, event_name: impl Into<std::string::String>) -> NodeHandle {
        let event_name = event_name.into();
        let node_name = self.unique_name(&format!("CustomEvent_{event_name}"));
        let pos = self.next_pos();
        let node = nodes::custom_event(&node_name, &event_name).at(pos.x, pos.y);
        self.push_node(node)
    }

    // ------------------------------------------------------------------
    // Mutable-style action node additions
    // ------------------------------------------------------------------

    /// Add a PrintString node with a fixed string literal (mutable style).
    pub fn print_string(&mut self, text: impl Into<std::string::String>) -> NodeHandle {
        let text = text.into();
        let name = self.unique_name("PrintString");
        let pos = self.next_pos();
        let mut in_string = Pin::data_input("InString", PinType::string());
        in_string.default_value = Some(text);
        let node = BpNode::new(
            "/Script/BlueprintGraph.K2Node_CallFunction",
            &name,
        )
        .at(pos.x, pos.y)
        .with_property(
            "FunctionReference",
            "(MemberParent=Class'/Script/Engine.KismetSystemLibrary',MemberName=\"PrintString\")",
        )
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(in_string)
        .with_pin(Pin::data_input("bPrintToScreen", PinType::bool()).with_default("true"))
        .with_pin(Pin::data_input("bPrintToLog", PinType::bool()).with_default("true"));
        self.push_node(node)
    }

    /// Add a Branch (if-then-else) node (mutable style).
    pub fn branch_node(&mut self) -> NodeHandle {
        let name = self.unique_name("Branch");
        let pos = self.next_pos();
        let node = nodes::branch_node(&name).at(pos.x, pos.y);
        self.push_node(node)
    }

    /// Add an integer add node (mutable style).
    pub fn add_int(&mut self) -> NodeHandle {
        let name = self.unique_name("AddInt");
        let pos = self.next_pos();
        let node = nodes::add_int(&name).at(pos.x, pos.y);
        self.push_node(node)
    }

    /// Add a SetTimer by event node (mutable style).
    pub fn set_timer_by_event(&mut self, rate: f32, looping: bool) -> NodeHandle {
        let name = self.unique_name("SetTimerByEvent");
        let pos = self.next_pos();
        let looping_str = if looping { "true" } else { "false" };
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", &name)
            .at(pos.x, pos.y)
            .with_property(
                "FunctionReference",
                "(MemberParent=Class'/Script/Engine.GameplayStatics',MemberName=\"SetTimerByEvent\")",
            )
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(
                Pin::data_input("Time", PinType::float())
                    .with_default(&rate.to_string()),
            )
            .with_pin(
                Pin::data_input("bLooping", PinType::bool())
                    .with_default(looping_str),
            );
        self.push_node(node)
    }

    /// Add a SetActorLocation node (mutable style).
    pub fn set_actor_location(&mut self, x: f32, y: f32, z: f32) -> NodeHandle {
        let name = self.unique_name("SetActorLocation");
        let pos = self.next_pos();
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", &name)
            .at(pos.x, pos.y)
            .with_property(
                "FunctionReference",
                "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"K2_SetActorLocation\")",
            )
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(
                Pin::data_input(
                    "NewLocation",
                    PinType::struct_type("/Script/CoreUObject.Vector"),
                )
                .with_default(&format!("(X={x},Y={y},Z={z})")),
            );
        self.push_node(node)
    }

    /// Add an AddMovementInput node (mutable style, for Character Blueprints).
    pub fn add_movement_input(&mut self) -> NodeHandle {
        let name = self.unique_name("AddMovementInput");
        let pos = self.next_pos();
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", &name)
            .at(pos.x, pos.y)
            .with_property(
                "FunctionReference",
                "(MemberParent=Class'/Script/Engine.Pawn',MemberName=\"AddMovementInput\")",
            )
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_input(
                "WorldDirection",
                PinType::struct_type("/Script/CoreUObject.Vector"),
            ))
            .with_pin(
                Pin::data_input("ScaleValue", PinType::float()).with_default("1.0"),
            )
            .with_pin(
                Pin::data_input("bForce", PinType::bool()).with_default("false"),
            );
        self.push_node(node)
    }

    /// Add a BindEventToOnClicked node (mutable style).
    pub fn bind_event_on_clicked(
        &mut self,
        button_var: impl Into<std::string::String>,
    ) -> NodeHandle {
        let button_var = button_var.into();
        let name = self.unique_name("BindEventOnClicked");
        let pos = self.next_pos();
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", &name)
            .at(pos.x, pos.y)
            .with_property(
                "FunctionReference",
                "(MemberParent=Class'/Script/UMG.Button',MemberName=\"OnClicked\")",
            )
            .with_property("ButtonVar", &button_var)
            .with_pin(Pin::exec_input("execute"))
            .with_pin(Pin::exec_output("then"));
        self.push_node(node)
    }

    // ------------------------------------------------------------------
    // Connection helpers
    // ------------------------------------------------------------------

    /// Connect exec-output "then" of `from` to exec-input "execute" of `to`.
    pub fn exec_connect(&mut self, from: &NodeHandle, to: &NodeHandle) {
        self.connect(from, "then", to, "execute");
    }

    /// Connect an arbitrary pin on `from` to an arbitrary pin on `to`.
    pub fn connect(
        &mut self,
        from: &NodeHandle,
        from_pin: &str,
        to: &NodeHandle,
        to_pin: &str,
    ) {
        self.blueprint
            .event_graph()
            .connect(&from.name, from_pin, &to.name, to_pin);
    }

    // ------------------------------------------------------------------
    // Build / serialize
    // ------------------------------------------------------------------

    /// Consume the builder and return the finished [`Blueprint`].
    pub fn build(self) -> Blueprint {
        self.blueprint
    }

    /// Build and serialise to UE4 T3D format (returns a `String`).
    pub fn to_t3d(&self) -> std::string::String {
        T3dSerializer::serialize(&self.blueprint)
    }

    /// Build and serialise to pretty JSON.
    pub fn to_json(&self) -> Result<std::string::String, serde_json::Error> {
        JsonSerializer::serialize(&self.blueprint)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consuming_begin_play() {
        let t3d = BlueprintBuilder::new("TestActor", "Actor")
            .variable("Health", VarType::Int, Some("100".into()))
            .begin_play(|ev| {
                ev.print("Hello from blueprint-core!");
            })
            .to_t3d();

        assert!(t3d.contains("ReceiveBeginPlay"));
        assert!(t3d.contains("PrintString"));
    }

    #[test]
    fn consuming_tick() {
        let t3d = BlueprintBuilder::new("MyPawn", "Pawn")
            .tick(|ev| {
                ev.call("UpdateMovement", vec!["DeltaSeconds".into()]);
            })
            .to_t3d();

        assert!(t3d.contains("ReceiveTick"));
        assert!(t3d.contains("UpdateMovement"));
    }

    #[test]
    fn mutable_exec_connect() {
        let mut builder = BlueprintBuilder::new("MyActor", "Actor");
        let ev = builder.begin_play_node();
        let ps = builder.print_string("Hello!");
        builder.exec_connect(&ev, &ps);
        let t3d = builder.to_t3d();
        assert!(t3d.contains("ReceiveBeginPlay"));
        assert!(t3d.contains("PrintString"));
    }
}
