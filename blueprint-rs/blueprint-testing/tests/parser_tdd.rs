//! TDD integration tests for `blueprint_core::parser::generate_rust_code`.
//!
//! These tests use `chicago_tdd_tools::TestEnvironment` for setup/teardown
//! isolation and drive the parser's code-generator with hand-crafted T3D
//! snippets so every branch of `generate_rust_code` is covered.

use blueprint_core::ast::{BpGraph, BpNode, Pin};
use blueprint_core::parser::{generate_rust_code, parse_t3d};
use blueprint_core::serializer::T3dSerializer;
use blueprint_core::types::PinType;
use chicago_tdd_tools::TestEnvironment;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a single-node T3D blob from a hand-crafted BpNode.
fn t3d_for_single_node(node: BpNode) -> String {
    let mut graph = BpGraph::new("EventGraph");
    graph.add_node(node);
    T3dSerializer::serialize_graph(&graph)
}

/// Parse a T3D blob and run `generate_rust_code`. Panics on parse failure.
fn codegen(t3d: &str) -> anyhow::Result<String> {
    let nodes = parse_t3d(t3d).expect("T3D parse must succeed");
    Ok(generate_rust_code(&nodes, "TestBP", "Actor")?)
}

// ---------------------------------------------------------------------------
// Test 1: K2Node_Event with ReceiveTick generates tick_node
// ---------------------------------------------------------------------------

#[test]
fn event_receive_tick_generates_tick_node() {
    let _env = TestEnvironment::new().expect("test env setup failed");

    let node = BpNode::new("/Script/BlueprintGraph.K2Node_Event", "K2Node_Event_0")
        .with_property(
            "EventReference",
            "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"ReceiveTick\")",
        )
        .with_property("bOverrideFunction", "True")
        .with_pin(Pin::data_output("DeltaSeconds", PinType::float()));

    let t3d = t3d_for_single_node(node);
    let code = codegen(&t3d).expect("generate_rust_code must succeed for ReceiveTick");

    assert!(
        code.contains("tick_node"),
        "ReceiveTick should map to builder.tick_node(), got:\n{code}"
    );
    assert!(
        !code.contains("begin_play_node"),
        "ReceiveTick must NOT emit begin_play_node, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// Test 2: K2Node_Event with ReceiveEndPlay generates end_play_node
// ---------------------------------------------------------------------------

#[test]
fn event_receive_end_play_generates_end_play_node() {
    let _env = TestEnvironment::new().expect("test env setup failed");

    let node = BpNode::new("/Script/BlueprintGraph.K2Node_Event", "K2Node_Event_0")
        .with_property(
            "EventReference",
            "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"ReceiveEndPlay\")",
        )
        .with_property("bOverrideFunction", "True")
        .with_pin(Pin::exec_output("then"));

    let t3d = t3d_for_single_node(node);
    let code = codegen(&t3d).expect("generate_rust_code must succeed for ReceiveEndPlay");

    assert!(
        code.contains("end_play_node"),
        "ReceiveEndPlay should map to builder.end_play_node(), got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// Test 3: K2Node_CallFunction with PrintString generates print_string
// ---------------------------------------------------------------------------

#[test]
fn call_function_print_string_generates_print_string() {
    let _env = TestEnvironment::new().expect("test env setup failed");

    let mut in_str = Pin::data_input("InString", PinType::string());
    in_str.default_value = Some("Hello!".to_string());

    let node = BpNode::new(
        "/Script/BlueprintGraph.K2Node_CallFunction",
        "K2Node_CallFunction_0",
    )
    .with_property(
        "FunctionReference",
        "(MemberParent=Class'/Script/Engine.KismetSystemLibrary',MemberName=\"PrintString\")",
    )
    .with_pin(Pin::exec_input("execute"))
    .with_pin(Pin::exec_output("then"))
    .with_pin(in_str);

    let t3d = t3d_for_single_node(node);
    let code = codegen(&t3d).expect("generate_rust_code must succeed for PrintString");

    assert!(
        code.contains("print_string"),
        "PrintString should map to builder.print_string(), got:\n{code}"
    );
    assert!(
        code.contains("Hello!"),
        "The InString default value should appear in the generated code, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// Test 4: K2Node_CallFunction with ApplyDamage generates apply_damage_node
// ---------------------------------------------------------------------------

#[test]
fn call_function_apply_damage_generates_apply_damage_node() {
    let _env = TestEnvironment::new().expect("test env setup failed");

    let node = BpNode::new(
        "/Script/BlueprintGraph.K2Node_CallFunction",
        "K2Node_CallFunction_0",
    )
    .with_property(
        "FunctionReference",
        "(MemberParent=Class'/Script/Engine.GameplayStatics',MemberName=\"ApplyDamage\")",
    )
    .with_pin(Pin::exec_input("execute"))
    .with_pin(Pin::exec_output("then"));

    let t3d = t3d_for_single_node(node);
    let code = codegen(&t3d).expect("generate_rust_code must succeed for ApplyDamage");

    assert!(
        code.contains("apply_damage_node"),
        "ApplyDamage should map to builder.apply_damage_node(), got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// Test 5: K2Node_IfThenElse generates branch_node
// ---------------------------------------------------------------------------

#[test]
fn if_then_else_generates_branch_node() {
    let _env = TestEnvironment::new().expect("test env setup failed");

    let node = BpNode::new(
        "/Script/BlueprintGraph.K2Node_IfThenElse",
        "K2Node_IfThenElse_0",
    )
    .with_pin(Pin::exec_input("execute"))
    .with_pin(Pin::data_input("Condition", PinType::bool()))
    .with_pin(Pin::exec_output("then"))
    .with_pin(Pin::exec_output("else"));

    let t3d = t3d_for_single_node(node);
    let code = codegen(&t3d).expect("generate_rust_code must succeed for K2Node_IfThenElse");

    assert!(
        code.contains("branch_node"),
        "K2Node_IfThenElse should map to builder.branch_node(), got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Unknown event ref returns Err, not a broken comment
// ---------------------------------------------------------------------------

#[test]
fn unknown_event_ref_returns_err() {
    let _env = TestEnvironment::new().expect("test env setup failed");

    // A K2Node_Event with an EventReference the parser does not know about.
    let node = BpNode::new("/Script/BlueprintGraph.K2Node_Event", "K2Node_Event_0")
        .with_property(
            "EventReference",
            "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"ReceiveUnknownObscureEvent\")",
        )
        .with_property("bOverrideFunction", "True")
        .with_pin(Pin::exec_output("then"));

    let t3d = t3d_for_single_node(node);
    let nodes = parse_t3d(&t3d).expect("T3D parse must succeed");
    let result = generate_rust_code(&nodes, "TestBP", "Actor");

    assert!(
        result.is_err(),
        "An unknown EventReference should produce an Err, not a broken comment. \
         Got: {result:?}"
    );

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Unsupported K2Node_Event"),
        "Error message should mention 'Unsupported K2Node_Event', got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// Test 7: K2Node_VariableGet generates variable_get_node with correct name
// ---------------------------------------------------------------------------

#[test]
fn variable_get_node_with_correct_name() {
    let _env = TestEnvironment::new().expect("test env setup failed");

    // Build a VariableGet node the same way the variables module does it
    // (using VariableReference property with MemberName).
    let node = BpNode::new(
        "/Script/BlueprintGraph.K2Node_VariableGet",
        "K2Node_VariableGet_0",
    )
    .with_property(
        "VariableReference",
        "(MemberName=\"PlayerHealth\",bSelfContext=True)",
    )
    .with_pin(Pin::data_output("PlayerHealth", PinType::float()));

    let t3d = t3d_for_single_node(node);
    let code = codegen(&t3d).expect("generate_rust_code must succeed for K2Node_VariableGet");

    assert!(
        code.contains("variable_get_node"),
        "K2Node_VariableGet should map to builder.variable_get_node(), got:\n{code}"
    );
    assert!(
        code.contains("PlayerHealth"),
        "The variable name 'PlayerHealth' should appear in the generated code, got:\n{code}"
    );
}

// ---------------------------------------------------------------------------
// Property-based tests: node-handler registry is deterministic
// ---------------------------------------------------------------------------

proptest! {
    // Simple node types (no extra properties required) generate identical output
    // on repeated calls — the OnceLock registry must be idempotent
    #[test]
    fn registry_codegen_is_deterministic(class_idx in 0usize..3usize) {
        let entries: &[(&str, &str)] = &[
            ("/Script/BlueprintGraph.K2Node_IfThenElse",  "K2Node_IfThenElse_0"),
            ("/Script/BlueprintGraph.K2Node_Select",      "K2Node_Select_0"),
            ("/Script/BlueprintGraph.K2Node_ForEachLoop", "K2Node_ForEachLoop_0"),
        ];
        let (class, name) = entries[class_idx];
        let node = BpNode::new(class, name);
        let mut graph = BpGraph::new("TestGraph");
        graph.add_node(node);
        let t3d = T3dSerializer::serialize_graph(&graph);
        let nodes = parse_t3d(&t3d).expect("parse should succeed");
        let code1 = generate_rust_code(&nodes, "TestBP", "Actor")
            .expect("codegen should succeed for known class");
        let code2 = generate_rust_code(&nodes, "TestBP", "Actor")
            .expect("repeated codegen should succeed");
        prop_assert_eq!(&code1, &code2, "codegen must be deterministic for {}", class);
    }

    // Registry rejects unknown classes and returns a ParseError
    #[test]
    fn registry_rejects_unknown_class(suffix in "[A-Z][a-z]{3,8}") {
        let class = format!("/Script/BlueprintGraph.K2Node_Unknown_{suffix}");
        let node = BpNode::new(&class, "TestNode_0");
        let mut graph = BpGraph::new("TestGraph");
        graph.add_node(node);
        let t3d = T3dSerializer::serialize_graph(&graph);
        let nodes = parse_t3d(&t3d).expect("parse should succeed");
        let result = generate_rust_code(&nodes, "TestBP", "Actor");
        prop_assert!(result.is_err(), "unknown class should produce ParseError");
        let msg = result.unwrap_err().to_string();
        prop_assert!(msg.contains("Unsupported"), "error should mention 'Unsupported', got: {msg}");
    }
}
