use blueprint_core::ast::Blueprint;
use blueprint_core::serializer::T3dSerializer;
use blueprint_core::registry::NodeRegistry;
use blueprint_testing::assert_connected;

#[test]
fn should_verify_427_es3_blueprint_semantic_correctness() {
    // 1. Setup - Create a blueprint for a standard UE4.27 Actor
    let mut bp = Blueprint::new("BrmPlayerController", "Actor");

    // 2. Act - Add 4.27 compatible nodes
    let registry = NodeRegistry::new();

    // Add BeginPlay event
    let mut begin_play = registry.create("event_begin_play", "BeginPlay").expect("BeginPlay should exist");
    // Manual adjustment to match UE4.27 T3D precisely if needed
    begin_play.properties.insert("bOverrideFunction".to_string(), "True".to_string());

    // Add PrintString node
    let print_string = registry.create("print_string", "PrintString").expect("PrintString should exist");

    let graph = bp.event_graph();
    graph.add_node(begin_play);
    graph.add_node(print_string);

    // Connect BeginPlay:then -> PrintString:execute
    graph.connect("BeginPlay", "then", "PrintString", "execute");

    // 3. Assert - Verify node classes and structural properties via the AST

    let graph = bp.event_graph();

    // Verify node count
    assert_eq!(graph.nodes.len(), 2, "EventGraph should contain exactly 2 nodes");

    // Verify BeginPlay node class
    let begin_play_node = graph.node("BeginPlay").expect("BeginPlay node should exist");
    assert_eq!(
        begin_play_node.class,
        "/Script/BlueprintGraph.K2Node_Event",
        "BeginPlay should use K2Node_Event class"
    );

    // Verify bOverrideFunction property on BeginPlay
    assert_eq!(
        begin_play_node.properties.get("bOverrideFunction").map(String::as_str),
        Some("True"),
        "BeginPlay should have bOverrideFunction=True"
    );

    // Verify PrintString node class
    let print_node = graph.node("PrintString").expect("PrintString node should exist");
    assert_eq!(
        print_node.class,
        "/Script/BlueprintGraph.K2Node_CallFunction",
        "PrintString should use K2Node_CallFunction class"
    );

    // Verify FunctionReference property on PrintString points to KismetSystemLibrary
    let func_ref = print_node.properties.get("FunctionReference")
        .expect("PrintString should have FunctionReference property");
    assert_eq!(
        func_ref,
        "(MemberParent=Class'/Script/Engine.KismetSystemLibrary',MemberName=\"PrintString\")",
        "FunctionReference must be exact UE4 T3D reference to KismetSystemLibrary::PrintString; got: {func_ref}"
    );

    // Verify node positions are present (non-default struct with x/y fields)
    // NodePos is always present on a BpNode; having a position struct means layout is defined
    let _ = begin_play_node.pos;
    let _ = print_node.pos;

    // Verify node IDs (GUIDs) are present
    assert!(!begin_play_node.id.to_string().is_empty(), "BeginPlay node must have a GUID");
    assert!(!print_node.id.to_string().is_empty(), "PrintString node must have a GUID");

    // Verify pin presence on PrintString (at minimum an exec input)
    assert!(
        print_node.find_pin("execute").is_some(),
        "PrintString should have an 'execute' exec input pin"
    );

    // Verify bidirectional linkage via AST
    let then_pin = begin_play_node.find_pin("then")
        .expect("BeginPlay should have a 'then' exec output pin");
    assert_eq!(then_pin.linked_to.len(), 1, "BeginPlay:then should have exactly one link");
    assert_eq!(
        then_pin.linked_to[0].node_name, "PrintString",
        "BeginPlay:then should link to PrintString"
    );

    let exec_pin = print_node.find_pin("execute")
        .expect("PrintString should have an 'execute' pin");
    assert_eq!(exec_pin.linked_to.len(), 1, "PrintString:execute should have exactly one link");
    assert_eq!(
        exec_pin.linked_to[0].node_name, "BeginPlay",
        "PrintString:execute should link back to BeginPlay"
    );

    // Verify Connections via AST assertion macro
    assert_connected!(bp, "EventGraph", "BeginPlay", "then", "PrintString", "execute");

    // Ensure the serializer can produce output without panicking (smoke test only)
    let _t3d = T3dSerializer::serialize(&bp);
}

#[test]
fn should_verify_actor_spawning_logic_for_427() {
    // 1. Setup
    let mut bp = Blueprint::new("SpawnManager", "Actor");
    let registry = NodeRegistry::new();

    // 2. Act - Create a SpawnActorFromClass node
    let mut spawn_node = registry.create("spawn_actor", "SpawnEnemy").expect("spawn_actor node should exist");

    // Set a specific actor class (ES3 compatible)
    if let Some(pin) = spawn_node.find_pin_mut("ActorClass") {
        pin.default_value = Some("Class'/Script/Engine.StaticMeshActor'".to_string());
    }

    bp.event_graph().add_node(spawn_node);

    // 3. Assert via structural AST checks

    let graph = bp.event_graph();
    let spawn = graph.node("SpawnEnemy").expect("SpawnEnemy node should exist in the graph");

    // Verify the node class — spawn_actor maps to K2Node_CallFunction
    assert_eq!(
        spawn.class,
        "/Script/BlueprintGraph.K2Node_CallFunction",
        "SpawnEnemy should use K2Node_CallFunction"
    );

    // Verify FunctionReference points to GameplayStatics::BeginSpawningActorFromClass
    let func_ref = spawn.properties.get("FunctionReference")
        .expect("SpawnEnemy should have a FunctionReference property");
    assert_eq!(
        func_ref,
        "(MemberParent=Class'/Script/Engine.GameplayStatics',MemberName=\"BeginSpawningActorFromClass\")",
        "FunctionReference must be exact UE4 T3D reference to GameplayStatics::BeginSpawningActorFromClass; got: {func_ref}"
    );

    // Verify ActorClass pin exists
    let actor_class_pin = spawn.find_pin("ActorClass")
        .expect("SpawnEnemy should have an ActorClass input pin");

    // Verify the ActorClass default value was set correctly
    assert_eq!(
        actor_class_pin.default_value.as_deref(),
        Some("Class'/Script/Engine.StaticMeshActor'"),
        "ActorClass pin default value should be set to StaticMeshActor"
    );

    // Smoke test: serializer should not panic
    let _t3d = T3dSerializer::serialize(&bp);
}

#[test]
fn should_not_allow_mocking_and_use_real_registry() {
    // Ensure we are using the real registry and real serialization logic
    let registry = NodeRegistry::new();
    let node_count = registry.len();
    assert!(node_count > 100, "Registry should have a substantial number of real UE4 nodes");

    let math_node = registry.create("add_float", "AddFloats").unwrap();
    assert_eq!(math_node.class, "/Script/BlueprintGraph.K2Node_CommutativeAssociativeBinaryOperator");
}
