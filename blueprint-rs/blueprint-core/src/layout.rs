//! Auto-layout engine for Blueprint graphs.
//!
//! Implements a Sugiyama-inspired hierarchical (layered) layout algorithm
//! that positions nodes left-to-right: events on the left, terminal nodes
//! on the right. The algorithm runs in three phases:
//!
//! 1. **Layer assignment** — longest-path BFS from exec-flow source nodes.
//! 2. **Crossing minimisation** — barycenter heuristic within each layer.
//! 3. **Coordinate assignment** — fixed spacing with vertical centering.

use crate::ast::{Blueprint, BpGraph};
use crate::types::{NodePos, PinCategory, PinDirection};
use std::collections::{HashMap, VecDeque};

// ---------------------------------------------------------------------------
// LayoutConfig
// ---------------------------------------------------------------------------

/// Configuration for the auto-layout algorithm.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Horizontal spacing between layers (columns) in UE4 units.
    pub layer_spacing: i32,
    /// Vertical spacing between nodes in the same layer.
    pub node_spacing: i32,
    /// Starting X position (leftmost node column).
    pub origin_x: i32,
    /// Starting Y position (centre of the graph).
    pub origin_y: i32,
    /// Estimated node width for spacing calculations.
    pub node_width: i32,
    /// Estimated node height for spacing calculations.
    pub node_height: i32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            layer_spacing: 220,
            node_spacing: 150,
            origin_x: 0,
            origin_y: 0,
            node_width: 180,
            node_height: 120,
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Apply automatic layout to all graphs in a Blueprint using default config.
pub fn auto_layout_blueprint(blueprint: &mut Blueprint) {
    auto_layout_blueprint_with_config(blueprint, &LayoutConfig::default());
}

/// Apply automatic layout to all graphs in a Blueprint with a custom config.
pub fn auto_layout_blueprint_with_config(blueprint: &mut Blueprint, config: &LayoutConfig) {
    for graph in &mut blueprint.graphs {
        auto_layout_graph_with_config(graph, config);
    }
}

/// Apply automatic layout to a single Blueprint graph using default config.
pub fn auto_layout_graph(graph: &mut BpGraph) {
    auto_layout_graph_with_config(graph, &LayoutConfig::default());
}

/// Apply automatic layout to a single Blueprint graph with a custom config.
pub fn auto_layout_graph_with_config(graph: &mut BpGraph, config: &LayoutConfig) {
    if graph.nodes.is_empty() {
        return;
    }

    let n = graph.nodes.len();

    // Step 1: Build adjacency list from exec-flow connections only.
    // Exec flow determines the primary left-to-right ordering.
    let node_indices: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.name.as_str(), i))
        .collect();

    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); n];

    for (i, node) in graph.nodes.iter().enumerate() {
        for pin in &node.pins {
            if pin.direction == PinDirection::Output && pin.pin_type.category == PinCategory::Exec {
                for link in &pin.linked_to {
                    if let Some(&j) = node_indices.get(link.node_name.as_str()) {
                        if i != j {
                            successors[i].push(j);
                            predecessors[j].push(i);
                        }
                    }
                }
            }
        }
    }

    // Step 2: Assign layers using longest-path layering (BFS from sources).
    let layers = assign_layers(n, &successors, &predecessors);

    // Step 3: Group nodes by layer.
    let max_layer = layers.iter().copied().max().unwrap_or(0);
    let mut layer_nodes: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
    for (i, &layer) in layers.iter().enumerate() {
        layer_nodes[layer].push(i);
    }

    // Step 4: Within each layer, sort nodes by barycenter to minimise crossings.
    // We iterate forward (left to right) and sort each layer by the average
    // position of its predecessors in the previous layer.
    for layer in 1..=max_layer {
        // Snapshot previous layer positions before we mutate layer_nodes.
        let prev_positions: HashMap<usize, usize> = layer_nodes[layer - 1]
            .iter()
            .enumerate()
            .map(|(pos, &node)| (node, pos))
            .collect();

        layer_nodes[layer].sort_by_key(|&node_idx| {
            let preds = &predecessors[node_idx];
            if preds.is_empty() {
                // Nodes with no predecessors go to the bottom of the layer.
                usize::MAX
            } else {
                let sum: usize = preds
                    .iter()
                    .filter_map(|&p| prev_positions.get(&p).copied())
                    .sum();
                sum / preds.len().max(1)
            }
        });
    }

    // Step 5: Assign final (x, y) coordinates.
    let mut positions: Vec<NodePos> = vec![NodePos::default(); n];

    for (layer, nodes_in_layer) in layer_nodes.iter().enumerate() {
        let x = config.origin_x + layer as i32 * (config.node_width + config.layer_spacing);

        // Centre nodes vertically around origin_y.
        // For N nodes the "occupied span" is (N-1) steps; the first node is
        // placed so the group is symmetric about origin_y.
        let count = nodes_in_layer.len() as i32;
        let step = config.node_height + config.node_spacing;
        let total_span = (count - 1) * step; // distance from first to last node top-edge
        let start_y = config.origin_y - total_span / 2;

        for (row, &node_idx) in nodes_in_layer.iter().enumerate() {
            positions[node_idx] = NodePos {
                x,
                y: start_y + row as i32 * step,
            };
        }
    }

    // Step 6: Write positions back to the graph nodes.
    for (i, node) in graph.nodes.iter_mut().enumerate() {
        node.pos = positions[i].clone();
    }
}

// ---------------------------------------------------------------------------
// Convenience: layout + build
// ---------------------------------------------------------------------------

/// Run [`auto_layout_blueprint`] on the blueprint produced by the builder,
/// then return the finished [`Blueprint`].
pub fn layout_and_build(builder: crate::builder::BlueprintBuilder) -> crate::ast::Blueprint {
    let mut bp = builder.build();
    auto_layout_blueprint(&mut bp);
    bp
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Assign layers using Kahn's BFS topological sort with longest-path layering.
///
/// Each node's layer is at least `predecessor_layer + 1`, so the layer
/// corresponds to the length of the longest incoming exec-flow path.
fn assign_layers(n: usize, successors: &[Vec<usize>], predecessors: &[Vec<usize>]) -> Vec<usize> {
    let mut layers = vec![0usize; n];
    let mut in_degree: Vec<usize> = predecessors.iter().map(|p| p.len()).collect();
    let mut queue: VecDeque<usize> = VecDeque::new();

    // Seed the queue with all source nodes (no predecessors).
    for (i, &deg) in in_degree.iter().enumerate().take(n) {
        if deg == 0 {
            queue.push_back(i);
        }
    }

    while let Some(node) = queue.pop_front() {
        for &succ in &successors[node] {
            // Successor must be at least one layer ahead.
            if layers[succ] < layers[node] + 1 {
                layers[succ] = layers[node] + 1;
            }
            in_degree[succ] = in_degree[succ].saturating_sub(1);
            if in_degree[succ] == 0 {
                queue.push_back(succ);
            }
        }
    }

    layers
}

// ---------------------------------------------------------------------------
// Utility queries
// ---------------------------------------------------------------------------

/// Calculate the axis-aligned bounding box of all node positions in a graph.
///
/// Returns `(top_left, bottom_right)`. For an empty graph both points are the
/// default `NodePos` (0, 0).
pub fn bounding_box(graph: &BpGraph) -> (NodePos, NodePos) {
    if graph.nodes.is_empty() {
        return (NodePos::default(), NodePos::default());
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for node in &graph.nodes {
        min_x = min_x.min(node.pos.x);
        min_y = min_y.min(node.pos.y);
        max_x = max_x.max(node.pos.x);
        max_y = max_y.max(node.pos.y);
    }

    (
        NodePos { x: min_x, y: min_y },
        NodePos { x: max_x, y: max_y },
    )
}

/// Returns `true` if any two nodes overlap within their estimated bounding
/// rectangles (using `config.node_width` / `config.node_height`).
pub fn has_overlapping_nodes(graph: &BpGraph, config: &LayoutConfig) -> bool {
    for (i, a) in graph.nodes.iter().enumerate() {
        for b in graph.nodes.iter().skip(i + 1) {
            let dx = (a.pos.x - b.pos.x).abs();
            let dy = (a.pos.y - b.pos.y).abs();
            if dx < config.node_width && dy < config.node_height {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Blueprint, BpGraph, BpNode, Pin};

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    /// Build a graph with `count` unconnected nodes.
    fn make_isolated_graph(count: usize) -> BpGraph {
        let mut g = BpGraph::event_graph();
        for i in 0..count {
            g.add_node(BpNode::new("TestClass", format!("Node{i}")));
        }
        g
    }

    /// Build a linear exec chain: Node0 → Node1 → … → Node(count-1).
    fn make_chain(count: usize) -> BpGraph {
        let mut g = BpGraph::event_graph();
        for i in 0..count {
            let node = BpNode::new("TestClass", format!("Node{i}"))
                .with_pin(Pin::exec_input("execute"))
                .with_pin(Pin::exec_output("then"));
            g.add_node(node);
        }
        // Wire Node(i).then → Node(i+1).execute
        for i in 0..count - 1 {
            g.connect(
                &format!("Node{i}"),
                "then",
                &format!("Node{}", i + 1),
                "execute",
            );
        }
        g
    }

    // ------------------------------------------------------------------
    // 1. Empty graph doesn't panic
    // ------------------------------------------------------------------

    #[test]
    fn empty_graph_no_panic() {
        let mut g = BpGraph::event_graph();
        auto_layout_graph(&mut g); // must not panic
        assert!(g.nodes.is_empty());
    }

    // ------------------------------------------------------------------
    // 2. Single node stays at origin with default config
    // ------------------------------------------------------------------

    #[test]
    fn single_node_at_origin() {
        let mut g = make_isolated_graph(1);
        auto_layout_graph(&mut g);
        assert_eq!(g.nodes[0].pos.x, 0);
        assert_eq!(g.nodes[0].pos.y, 0);
    }

    // ------------------------------------------------------------------
    // 3. Two-node chain: second node one column to the right
    // ------------------------------------------------------------------

    #[test]
    fn two_node_chain_positions() {
        let mut g = make_chain(2);
        let config = LayoutConfig::default();
        auto_layout_graph_with_config(&mut g, &config);

        let n0 = g.node("Node0").unwrap();
        let n1 = g.node("Node1").unwrap();

        assert_eq!(n0.pos.x, config.origin_x);
        let expected_x = config.origin_x + config.node_width + config.layer_spacing;
        assert_eq!(n1.pos.x, expected_x);
    }

    // ------------------------------------------------------------------
    // 4. Three-node chain: layers are 0, 1, 2
    // ------------------------------------------------------------------

    #[test]
    fn three_node_chain_layers() {
        let mut g = make_chain(3);
        let config = LayoutConfig::default();
        auto_layout_graph_with_config(&mut g, &config);

        let col = |idx: usize| -> i32 {
            config.origin_x + idx as i32 * (config.node_width + config.layer_spacing)
        };

        assert_eq!(g.node("Node0").unwrap().pos.x, col(0));
        assert_eq!(g.node("Node1").unwrap().pos.x, col(1));
        assert_eq!(g.node("Node2").unwrap().pos.x, col(2));
    }

    // ------------------------------------------------------------------
    // 5. Parallel branches: same layer → different y positions
    // ------------------------------------------------------------------

    #[test]
    fn parallel_branches_different_y() {
        // Node0 → Node1
        //       → Node2
        let mut g = BpGraph::event_graph();
        for name in &["Node0", "Node1", "Node2"] {
            let node = BpNode::new("TestClass", *name)
                .with_pin(Pin::exec_input("execute"))
                .with_pin(Pin::exec_output("then"));
            g.add_node(node);
        }
        g.connect("Node0", "then", "Node1", "execute");
        g.connect("Node0", "then", "Node2", "execute");

        auto_layout_graph(&mut g);

        let y1 = g.node("Node1").unwrap().pos.y;
        let y2 = g.node("Node2").unwrap().pos.y;
        assert_ne!(
            y1, y2,
            "Parallel nodes in the same layer must have different y"
        );
    }

    // ------------------------------------------------------------------
    // 6. No overlapping nodes after layout
    // ------------------------------------------------------------------

    #[test]
    fn no_overlapping_nodes_chain() {
        let mut g = make_chain(5);
        let config = LayoutConfig::default();
        auto_layout_graph_with_config(&mut g, &config);
        assert!(
            !has_overlapping_nodes(&g, &config),
            "Nodes must not overlap after layout"
        );
    }

    #[test]
    fn no_overlapping_nodes_isolated() {
        let mut g = make_isolated_graph(6);
        let config = LayoutConfig::default();
        auto_layout_graph_with_config(&mut g, &config);
        assert!(
            !has_overlapping_nodes(&g, &config),
            "Isolated nodes must not overlap after layout"
        );
    }

    // ------------------------------------------------------------------
    // 7. bounding_box returns correct min/max
    // ------------------------------------------------------------------

    #[test]
    fn bounding_box_empty() {
        let g = BpGraph::event_graph();
        let (tl, br) = bounding_box(&g);
        assert_eq!(tl.x, 0);
        assert_eq!(tl.y, 0);
        assert_eq!(br.x, 0);
        assert_eq!(br.y, 0);
    }

    #[test]
    fn bounding_box_correct() {
        let mut g = make_chain(3);
        let config = LayoutConfig::default();
        auto_layout_graph_with_config(&mut g, &config);

        let (tl, br) = bounding_box(&g);
        // First column is leftmost
        assert_eq!(tl.x, config.origin_x);
        // Last column is rightmost
        let rightmost = config.origin_x + 2 * (config.node_width + config.layer_spacing);
        assert_eq!(br.x, rightmost);
    }

    // ------------------------------------------------------------------
    // 8. LayoutConfig::default() has reasonable values
    // ------------------------------------------------------------------

    #[test]
    fn default_config_reasonable() {
        let cfg = LayoutConfig::default();
        assert!(cfg.layer_spacing > 0, "layer_spacing must be positive");
        assert!(cfg.node_spacing > 0, "node_spacing must be positive");
        assert!(cfg.node_width > 0, "node_width must be positive");
        assert!(cfg.node_height > 0, "node_height must be positive");
        // Spacing should be at least somewhat meaningful for UE4
        assert!(cfg.layer_spacing >= 100);
        assert!(cfg.node_spacing >= 50);
    }

    // ------------------------------------------------------------------
    // 9. auto_layout_blueprint applies to all graphs
    // ------------------------------------------------------------------

    #[test]
    fn layout_blueprint_all_graphs() {
        let mut bp = Blueprint::new("TestActor", "Actor");
        // EventGraph is added automatically; add one more.
        {
            let fg = bp.function_graph("MyFunction");
            for i in 0..3 {
                fg.add_node(BpNode::new("TestClass", format!("FNode{i}")));
            }
        }
        // Also add nodes to the EventGraph.
        {
            let eg = bp.event_graph();
            eg.add_node(BpNode::new("TestClass", "EvNode0"));
            eg.add_node(BpNode::new("TestClass", "EvNode1"));
        }

        auto_layout_blueprint(&mut bp);

        // Verify isolated nodes in function graph are not overlapping.
        let fg = bp.graphs.iter().find(|g| g.name == "MyFunction").unwrap();
        let config = LayoutConfig::default();
        assert!(!has_overlapping_nodes(fg, &config));
    }

    // ------------------------------------------------------------------
    // 10. layout_and_build returns a Blueprint with laid-out nodes
    // ------------------------------------------------------------------

    #[test]
    fn layout_and_build_works() {
        use crate::builder::BlueprintBuilder;

        let bp = layout_and_build(BlueprintBuilder::new("TestActor", "Actor"));
        // Blueprint is returned without panic.
        assert_eq!(bp.name, "TestActor");
    }

    // ------------------------------------------------------------------
    // 11. Diamond graph (fork + join): layers assigned correctly
    // ------------------------------------------------------------------

    #[test]
    fn diamond_graph_layers() {
        //  A → B → D
        //    → C → D  (D has two predecessors)
        let mut g = BpGraph::event_graph();
        for name in &["A", "B", "C", "D"] {
            let node = BpNode::new("TestClass", *name)
                .with_pin(Pin::exec_input("execute"))
                .with_pin(Pin::exec_output("then"));
            g.add_node(node);
        }
        g.connect("A", "then", "B", "execute");
        g.connect("A", "then", "C", "execute");
        g.connect("B", "then", "D", "execute");
        g.connect("C", "then", "D", "execute");

        let config = LayoutConfig::default();
        auto_layout_graph_with_config(&mut g, &config);

        let col = |idx: i32| config.origin_x + idx * (config.node_width + config.layer_spacing);

        // A is at layer 0, B and C at layer 1, D at layer 2.
        assert_eq!(g.node("A").unwrap().pos.x, col(0));
        assert_eq!(g.node("B").unwrap().pos.x, col(1));
        assert_eq!(g.node("C").unwrap().pos.x, col(1));
        assert_eq!(g.node("D").unwrap().pos.x, col(2));
        // B and C must not overlap.
        assert_ne!(g.node("B").unwrap().pos.y, g.node("C").unwrap().pos.y);
    }

    // ------------------------------------------------------------------
    // 12. has_overlapping_nodes detects actual overlaps
    // ------------------------------------------------------------------

    #[test]
    fn overlap_detection_works() {
        let mut g = BpGraph::event_graph();
        // Place two nodes at the same position.
        let mut n0 = BpNode::new("TestClass", "N0");
        let mut n1 = BpNode::new("TestClass", "N1");
        n0.pos = NodePos::new(0, 0);
        n1.pos = NodePos::new(0, 0);
        g.add_node(n0);
        g.add_node(n1);

        let config = LayoutConfig::default();
        assert!(
            has_overlapping_nodes(&g, &config),
            "Stacked nodes must be detected as overlapping"
        );
    }

    // ------------------------------------------------------------------
    // 13. Custom config is respected
    // ------------------------------------------------------------------

    #[test]
    fn custom_config_respected() {
        let mut g = make_chain(2);
        let config = LayoutConfig {
            layer_spacing: 500,
            node_width: 300,
            origin_x: 100,
            origin_y: 200,
            ..LayoutConfig::default()
        };
        auto_layout_graph_with_config(&mut g, &config);

        let n0 = g.node("Node0").unwrap();
        let n1 = g.node("Node1").unwrap();

        assert_eq!(n0.pos.x, 100);
        assert_eq!(n1.pos.x, 100 + 300 + 500); // origin_x + node_width + layer_spacing
    }
}
