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

    // 3. Assert - Verify T3D output matches UE4.27/ES3 expectations
    let t3d = T3dSerializer::serialize(&bp);
    
    // Verify Node Classes (standard for 4.27)
    assert!(t3d.contains("Begin Object Class=/Script/BlueprintGraph.K2Node_Event Name=\"BeginPlay\""), "Missing BeginPlay node or incorrect class");
    assert!(t3d.contains("Begin Object Class=/Script/BlueprintGraph.K2Node_CallFunction Name=\"PrintString\""), "Missing PrintString node or incorrect class");
    
    // Verify ES3 compatible function references
    assert!(t3d.contains("FunctionReference=(MemberParent=Class'/Script/Engine.KismetSystemLibrary',MemberName=\"PrintString\")"), "Missing or incorrect FunctionReference for PrintString");
    assert!(t3d.contains("bOverrideFunction=True"), "Missing bOverrideFunction property on BeginPlay event");
    
    // Verify Layout Format
    assert!(t3d.contains("NodePosX="), "T3D output should specify node X position");
    assert!(t3d.contains("NodePosY="), "T3D output should specify node Y position");
    assert!(t3d.contains("NodeGuid="), "T3D output should specify node GUIDs");
    assert!(t3d.contains("CustomProperties Pin"), "T3D output should define custom pin properties");
    
    // Verify Linkage Layout in Pin format: LinkedTo=(PrintString(<pin_guid>))
    assert!(t3d.contains("LinkedTo=(PrintString("), "T3D should serialize the connection from BeginPlay to PrintString");
    assert!(t3d.contains("LinkedTo=(BeginPlay("), "T3D should serialize the connection back from PrintString to BeginPlay");

    // Verify Connections via AST assertion macro
    assert_connected!(bp, "EventGraph", "BeginPlay", "then", "PrintString", "execute");
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
    
    // 3. Assert
    let t3d = T3dSerializer::serialize(&bp);
    
    // Verify the actor class is correctly serialized for 4.27
    assert!(t3d.contains("Begin Object Class=/Script/BlueprintGraph.K2Node_CallFunction Name=\"SpawnEnemy\""), "Missing SpawnEnemy node");
    assert!(t3d.contains("FunctionReference=(MemberParent=Class'/Script/Engine.GameplayStatics',MemberName=\"BeginSpawningActorFromClass\")"), "Incorrect function reference for SpawnActor");
    assert!(t3d.contains("PinName=\"ActorClass\""), "Spawn node should have an ActorClass pin");
    assert!(t3d.contains("DefaultValue=\"Class'/Script/Engine.StaticMeshActor'\""), "ActorClass default value should be set to StaticMeshActor");
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
