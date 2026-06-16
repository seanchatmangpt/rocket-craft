/// BlueprintBuilder — fluent builder for constructing Blueprint graphs
///
/// Wraps a `Blueprint` and provides ergonomic methods for adding nodes and
/// wiring them together. All nodes are added to the EventGraph by default.

use crate::ast::{BpNode, BpVariable, Blueprint, Pin};
use crate::nodes::{begin_play_node, branch_node, add_int_node};
use crate::serializer::{T3dSerializer, JsonSerializer};
use crate::types::{NodePos, PinType};

/// A lightweight handle returned by builder node methods so callers can wire nodes
/// without holding references into the builder.
#[derive(Debug, Clone)]
pub struct NodeHandle {
    pub name: String,
}

impl NodeHandle {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

pub struct BlueprintBuilder {
    blueprint: Blueprint,
    /// Auto-incrementing counter used to generate unique node names
    node_counter: u32,
    /// X position for the next node (auto-layout)
    next_x: i32,
}

impl BlueprintBuilder {
    pub fn new(name: impl Into<String>, parent_class: impl Into<String>) -> Self {
        Self {
            blueprint: Blueprint::new(name, parent_class),
            node_counter: 0,
            next_x: 0,
        }
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    fn unique_name(&mut self, base: &str) -> String {
        self.node_counter += 1;
        format!("{}_{}", base, self.node_counter)
    }

    fn next_pos(&mut self) -> NodePos {
        let x = self.next_x;
        self.next_x += 250;
        NodePos::new(x, 0)
    }

    fn add_to_event_graph(&mut self, node: BpNode) -> NodeHandle {
        let name = node.name.clone();
        self.blueprint.event_graph().nodes.push(node);
        NodeHandle::new(name)
    }

    // ------------------------------------------------------------------
    // Variable management
    // ------------------------------------------------------------------

    pub fn variable(&mut self, var: BpVariable) -> &mut Self {
        self.blueprint.add_variable(var);
        self
    }

    // ------------------------------------------------------------------
    // Event nodes
    // ------------------------------------------------------------------

    /// Add a BeginPlay event node to the EventGraph
    pub fn begin_play(&mut self) -> NodeHandle {
        let name = self.unique_name("BeginPlay");
        let pos = self.next_pos();
        let node = begin_play_node(&name).at(pos.x, pos.y);
        self.add_to_event_graph(node)
    }

    /// Add an EndPlay event node
    pub fn end_play(&mut self) -> NodeHandle {
        let name = self.unique_name("EndPlay");
        let pos = self.next_pos();
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_Event", &name)
            .at(pos.x, pos.y)
            .with_property("EventReference",
                "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"ReceiveEndPlay\")")
            .with_property("bOverrideFunction", "True")
            .with_pin(Pin::exec_output("then"));
        self.add_to_event_graph(node)
    }

    /// Add a Tick event node
    pub fn tick(&mut self) -> NodeHandle {
        let name = self.unique_name("Tick");
        let pos = self.next_pos();
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_Event", &name)
            .at(pos.x, pos.y)
            .with_property("EventReference",
                "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"ReceiveTick\")")
            .with_property("bOverrideFunction", "True")
            .with_pin(Pin::exec_output("then"))
            .with_pin(Pin::data_output("DeltaSeconds", PinType::float()));
        self.add_to_event_graph(node)
    }

    /// Add a custom event node (K2Node_CustomEvent)
    pub fn custom_event(&mut self, event_name: impl Into<String>) -> NodeHandle {
        let event_name = event_name.into();
        let node_name = self.unique_name(&format!("CustomEvent_{}", event_name));
        let pos = self.next_pos();
        let node = BpNode::new("/Script/BlueprintGraph.K2Node_CustomEvent", &node_name)
            .at(pos.x, pos.y)
            .with_property("CustomFunctionName", &event_name)
            .with_pin(Pin::exec_output("then"));
        self.add_to_event_graph(node)
    }

    // ------------------------------------------------------------------
    // Action / function call nodes
    // ------------------------------------------------------------------

    /// Add a PrintString node with a fixed string literal
    pub fn print_string(&mut self, text: impl Into<String>) -> NodeHandle {
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
        self.add_to_event_graph(node)
    }

    /// Add a Branch (if-then-else) node
    pub fn branch(&mut self) -> NodeHandle {
        let name = self.unique_name("Branch");
        let pos = self.next_pos();
        let node = branch_node(&name).at(pos.x, pos.y);
        self.add_to_event_graph(node)
    }

    /// Add an integer add node
    pub fn add_int(&mut self) -> NodeHandle {
        let name = self.unique_name("AddInt");
        let pos = self.next_pos();
        let node = add_int_node(&name).at(pos.x, pos.y);
        self.add_to_event_graph(node)
    }

    /// Add a SetTimer by event node
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
            .with_pin(Pin::data_input("Time", PinType::float()).with_default(&rate.to_string()))
            .with_pin(Pin::data_input("bLooping", PinType::bool()).with_default(looping_str));
        self.add_to_event_graph(node)
    }

    /// Add a SetActorLocation node
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
                Pin::data_input("NewLocation", PinType::struct_type("/Script/CoreUObject.Vector"))
                    .with_default(&format!("(X={},Y={},Z={})", x, y, z)),
            );
        self.add_to_event_graph(node)
    }

    /// Add an AddMovementInput node (for Character Blueprints)
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
            .with_pin(Pin::data_input("ScaleValue", PinType::float()).with_default("1.0"))
            .with_pin(Pin::data_input("bForce", PinType::bool()).with_default("false"));
        self.add_to_event_graph(node)
    }

    /// Add a BindEventToOnClicked node for UMG button binding
    pub fn bind_event_on_clicked(&mut self, button_var: impl Into<String>) -> NodeHandle {
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
        self.add_to_event_graph(node)
    }

    // ------------------------------------------------------------------
    // Connection helpers
    // ------------------------------------------------------------------

    /// Connect the exec-output "then" of `from` to the exec-input "execute" of `to`.
    /// This is the most common wiring pattern for sequential execution.
    pub fn exec_connect(&mut self, from: &NodeHandle, to: &NodeHandle) {
        self.connect(from, "then", to, "execute");
    }

    /// Connect an arbitrary pin on `from_node` to an arbitrary pin on `to_node`.
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

    /// Consume the builder and return the finished `Blueprint`.
    pub fn build(self) -> Blueprint {
        self.blueprint
    }

    /// Build and serialize to T3D format.
    pub fn to_t3d(&self) -> String {
        T3dSerializer::serialize(&self.blueprint)
    }

    /// Build and serialize to pretty JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        JsonSerializer::serialize(&self.blueprint)
    }
}
