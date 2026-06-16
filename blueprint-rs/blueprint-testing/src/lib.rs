//! blueprint-testing — snapshot/assertion test framework for Blueprint graphs.
//!
//! Provides macros for asserting graph properties and snapshot-based regression
//! testing of T3D output.

use blueprint_core::ast::{Blueprint, BpGraph, BpNode, Pin};
use blueprint_core::types::{PinType, PinDirection, PinCategory, ContainerType};
use blueprint_core::serializer::T3dSerializer;
use blueprint_core::validator;

// ============================================================
// ASSERTION MACROS
// ============================================================

/// Assert that a node with the given name exists in `$graph` of `$blueprint`.
///
/// # Example
/// ```
/// use blueprint_testing::assert_has_node;
/// let bp = blueprint_testing::minimal_blueprint("Test");
/// assert_has_node!(bp, "EventGraph", "BeginPlay");
/// ```
#[macro_export]
macro_rules! assert_has_node {
    ($blueprint:expr, $graph:expr, $node_name:expr) => {{
        let graph_name: &str = $graph;
        let node_name: &str = $node_name;
        let graph = $blueprint
            .graphs
            .iter()
            .find(|g| g.name == graph_name)
            .unwrap_or_else(|| panic!("Graph '{}' not found in blueprint", graph_name));
        let found = graph.nodes.iter().any(|n| n.name == node_name);
        assert!(
            found,
            "Expected node '{}' in graph '{}', but it was not found. Nodes present: [{}]",
            node_name,
            graph_name,
            graph.nodes.iter().map(|n| n.name.as_str()).collect::<Vec<_>>().join(", ")
        );
    }};
}

/// Assert that two nodes are directly connected via specific pins.
///
/// Checks that `$from_node.$from_pin` has a link pointing to `$to_node.$to_pin`.
///
/// # Example
/// ```
/// use blueprint_testing::assert_connected;
/// // (after setting up blueprint with connected nodes)
/// ```
#[macro_export]
macro_rules! assert_connected {
    ($blueprint:expr, $graph:expr, $from_node:expr, $from_pin:expr, $to_node:expr, $to_pin:expr) => {{
        let graph_name: &str = $graph;
        let from_node: &str = $from_node;
        let from_pin: &str = $from_pin;
        let to_node: &str = $to_node;
        let to_pin: &str = $to_pin;

        let graph = $blueprint
            .graphs
            .iter()
            .find(|g| g.name == graph_name)
            .unwrap_or_else(|| panic!("Graph '{}' not found in blueprint", graph_name));

        let src_node = graph
            .nodes
            .iter()
            .find(|n| n.name == from_node)
            .unwrap_or_else(|| panic!("Source node '{}' not found in graph '{}'", from_node, graph_name));

        let src_pin = src_node
            .pins
            .iter()
            .find(|p| p.name == from_pin)
            .unwrap_or_else(|| panic!("Pin '{}' not found on node '{}'", from_pin, from_node));

        let dest_node = graph
            .nodes
            .iter()
            .find(|n| n.name == to_node)
            .unwrap_or_else(|| panic!("Destination node '{}' not found in graph '{}'", to_node, graph_name));

        let dest_pin = dest_node
            .pins
            .iter()
            .find(|p| p.name == to_pin)
            .unwrap_or_else(|| panic!("Pin '{}' not found on node '{}'", to_pin, to_node));

        // Check that src_pin has a link to dest_node with dest_pin's id
        let connected = src_pin
            .linked_to
            .iter()
            .any(|link| link.node_name == to_node && link.pin_id == dest_pin.id);

        assert!(
            connected,
            "Expected pin '{}::{}' to be connected to '{}::{}', but no such connection exists. \
             Existing links from '{}::{}': [{}]",
            from_node, from_pin, to_node, to_pin,
            from_node, from_pin,
            src_pin.linked_to.iter()
                .map(|l| format!("{}({})", l.node_name, l.pin_id))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }};
}

/// Assert that the blueprint has no validation errors.
///
/// # Example
/// ```
/// use blueprint_testing::assert_no_validation_errors;
/// let bp = blueprint_testing::minimal_blueprint("Test");
/// assert_no_validation_errors!(bp);
/// ```
#[macro_export]
macro_rules! assert_no_validation_errors {
    ($blueprint:expr) => {{
        let errors = blueprint_core::validator::validate(&$blueprint);
        assert!(
            errors.is_empty(),
            "Expected no validation errors, but found {}:\n{}",
            errors.len(),
            blueprint_core::validator::format_errors(&errors)
        );
    }};
}

/// Assert that the T3D output of the blueprint contains a specific substring.
///
/// # Example
/// ```
/// use blueprint_testing::assert_t3d_contains;
/// let bp = blueprint_testing::minimal_blueprint("Test");
/// assert_t3d_contains!(bp, "Begin Object");
/// ```
#[macro_export]
macro_rules! assert_t3d_contains {
    ($blueprint:expr, $needle:expr) => {{
        let t3d = blueprint_core::serializer::T3dSerializer::serialize(&$blueprint);
        let needle: &str = $needle;
        assert!(
            t3d.contains(needle),
            "Expected T3D output to contain {:?}, but it did not.\n\nFull T3D output:\n{}",
            needle, t3d
        );
    }};
}

// ============================================================
// SNAPSHOT TESTING
// ============================================================

/// Save a Blueprint's T3D output as a snapshot file.
/// File is written to `tests/snapshots/<name>.t3d` relative to the
/// current directory (CARGO_MANIFEST_DIR when run under `cargo test`).
pub fn save_snapshot(blueprint: &Blueprint, name: &str) -> std::io::Result<()> {
    let snapshot_dir = snapshot_dir();
    std::fs::create_dir_all(&snapshot_dir)?;
    let path = snapshot_dir.join(format!("{}.t3d", name));
    let t3d = T3dSerializer::serialize(blueprint);
    std::fs::write(path, t3d)
}

/// Assert a Blueprint's T3D output matches a previously saved snapshot.
/// If no snapshot exists yet, saves it and passes (first-run mode).
///
/// Panics if an existing snapshot doesn't match.
pub fn assert_snapshot(blueprint: &Blueprint, name: &str) {
    let snapshot_dir = snapshot_dir();
    let path = snapshot_dir.join(format!("{}.t3d", name));
    let current_t3d = T3dSerializer::serialize(blueprint);

    if path.exists() {
        let saved = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read snapshot '{}': {}", path.display(), e));
        assert_eq!(
            current_t3d, saved,
            "Snapshot mismatch for '{}'. \
             If this is intentional, delete {} and re-run tests to update the snapshot.",
            name,
            path.display()
        );
    } else {
        // First run: create the snapshot
        std::fs::create_dir_all(&snapshot_dir)
            .unwrap_or_else(|e| panic!("Failed to create snapshot dir: {}", e));
        std::fs::write(&path, &current_t3d)
            .unwrap_or_else(|e| panic!("Failed to write snapshot '{}': {}", path.display(), e));
    }
}

/// Returns the path to the snapshots directory, resolving relative to
/// CARGO_MANIFEST_DIR when set (during `cargo test`), else the current dir.
fn snapshot_dir() -> std::path::PathBuf {
    let base = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    base.join("tests").join("snapshots")
}

// ============================================================
// TEST HELPERS
// ============================================================

/// Build a minimal valid Blueprint with one BeginPlay node.
///
/// Useful as a ready-made fixture in tests that just need a valid, non-empty blueprint.
pub fn minimal_blueprint(name: &str) -> Blueprint {
    let mut bp = Blueprint::new(name, "Actor");
    let begin_play_node = make_begin_play_node("BeginPlay");
    bp.event_graph().add_node(begin_play_node);
    bp
}

/// Build a Blueprint with two nodes connected via exec pins.
///
/// Creates a BeginPlay event and a PrintString function node connected by exec flow.
pub fn blueprint_from_t3d_or_builder(name: &str) -> Blueprint {
    let mut bp = Blueprint::new(name, "Actor");

    let begin_play = make_begin_play_node("BeginPlay");
    let print_node = make_print_node("PrintString");

    let graph = bp.event_graph();
    graph.add_node(begin_play);
    graph.add_node(print_node);
    graph.connect("BeginPlay", "then", "PrintString", "execute");

    bp
}

// ------ internal helpers -------------------------------------------------------

fn make_begin_play_node(name: &str) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_Event", name)
        .with_property(
            "EventReference",
            "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"ReceiveBeginPlay\")",
        )
        .with_property("bOverrideFunction", "True")
        .with_pin({
            let mut p = Pin::new("OutputDelegate", PinDirection::Output, PinType {
                category: PinCategory::Delegate,
                sub_category: None,
                sub_category_object: None,
                container: ContainerType::None,
                is_reference: false,
                is_const: false,
            });
            p.is_hidden = true;
            p.is_not_connectable = true;
            p
        })
        .with_pin(Pin::exec_output("then"))
}

fn make_print_node(name: &str) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", name)
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(Pin::data_input("InString", PinType::string()))
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // 1. assert_has_node passes for a node that exists
    #[test]
    fn test_assert_has_node_passes() {
        let bp = minimal_blueprint("TestBP");
        assert_has_node!(bp, "EventGraph", "BeginPlay");
    }

    // 2. assert_has_node panics when the node does not exist
    #[test]
    #[should_panic(expected = "Expected node 'NonExistentNode'")]
    fn test_assert_has_node_fails() {
        let bp = minimal_blueprint("TestBP");
        assert_has_node!(bp, "EventGraph", "NonExistentNode");
    }

    // 3. assert_connected passes for genuinely connected nodes
    #[test]
    fn test_assert_connected_passes() {
        let bp = blueprint_from_t3d_or_builder("ConnectedBP");
        assert_connected!(bp, "EventGraph", "BeginPlay", "then", "PrintString", "execute");
    }

    // 4. assert_no_validation_errors passes for a clean blueprint
    #[test]
    fn test_assert_no_validation_errors_clean() {
        let bp = minimal_blueprint("CleanBP");
        assert_no_validation_errors!(bp);
    }

    // 5. assert_t3d_contains passes when the T3D contains "Begin Object"
    #[test]
    fn test_assert_t3d_contains_passes() {
        let bp = minimal_blueprint("T3dBP");
        assert_t3d_contains!(bp, "Begin Object");
    }

    // 6. save_snapshot creates a file
    #[test]
    fn test_snapshot_creates_file() {
        let bp = minimal_blueprint("SnapshotBP");
        let snap_name = "test_snapshot_creates_file";

        // Clean up before test in case a previous run left a file
        let snap_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        let snap_path = snap_dir.join("tests").join("snapshots").join(format!("{}.t3d", snap_name));
        let _ = std::fs::remove_file(&snap_path);

        save_snapshot(&bp, snap_name).expect("save_snapshot should not fail");
        assert!(snap_path.exists(), "Snapshot file should have been created at {:?}", snap_path);

        // Cleanup
        let _ = std::fs::remove_file(&snap_path);
    }

    // 7. assert_snapshot matches on second call (round-trip stability)
    #[test]
    fn test_snapshot_matches() {
        let bp = minimal_blueprint("MatchBP");
        let snap_name = "test_snapshot_matches";

        // Clean up before test
        let snap_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        let snap_path = snap_dir.join("tests").join("snapshots").join(format!("{}.t3d", snap_name));
        let _ = std::fs::remove_file(&snap_path);

        // First call: creates the snapshot
        assert_snapshot(&bp, snap_name);
        assert!(snap_path.exists(), "Snapshot should exist after first call");

        // Second call: should match without panicking
        assert_snapshot(&bp, snap_name);

        // Cleanup
        let _ = std::fs::remove_file(&snap_path);
    }

    // 8. minimal_blueprint returns a valid blueprint with exactly one BeginPlay node
    #[test]
    fn test_minimal_blueprint_helper() {
        let bp = minimal_blueprint("MinimalBP");
        assert_eq!(bp.name, "MinimalBP");
        assert_eq!(bp.parent_class, "Actor");

        let event_graph = bp.graphs.iter().find(|g| g.name == "EventGraph")
            .expect("EventGraph must exist");
        let has_begin_play = event_graph.nodes.iter().any(|n| n.name == "BeginPlay");
        assert!(has_begin_play, "minimal_blueprint should include a BeginPlay node");

        // Verify no validation errors
        let errors = validator::validate(&bp);
        assert!(errors.is_empty(), "minimal_blueprint should produce a valid blueprint: {:?}", errors);
    }
}
