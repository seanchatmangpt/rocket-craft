use crate::ast::*;
use std::collections::{HashMap, HashSet};

// ============================================================
// DIFF TYPES
// ============================================================

/// A change to a single property on a node
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyChange {
    pub key: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// A change to a single pin
#[derive(Debug, Clone, PartialEq)]
pub struct PinChange {
    pub pin_name: String,
    pub kind: PinChangeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PinChangeKind {
    Added,
    Removed,
    DefaultValueChanged {
        before: Option<String>,
        after: Option<String>,
    },
    ConnectionAdded {
        to_node: String,
        to_pin: String,
    },
    ConnectionRemoved {
        to_node: String,
        to_pin: String,
    },
    TypeChanged {
        before: String,
        after: String,
    },
}

/// All changes to a single node
#[derive(Debug, Clone)]
pub struct NodeDiff {
    pub node_name: String,
    pub kind: NodeDiffKind,
}

#[derive(Debug, Clone)]
pub enum NodeDiffKind {
    Added(BpNode),
    Removed(BpNode),
    Modified {
        position_changed: bool,
        property_changes: Vec<PropertyChange>,
        pin_changes: Vec<PinChange>,
    },
}

/// All changes between two Blueprint graphs
#[derive(Debug, Clone, Default)]
pub struct GraphDiff {
    pub graph_name: String,
    pub node_diffs: Vec<NodeDiff>,
}

/// All changes between two Blueprints
#[derive(Debug, Clone, Default)]
pub struct BlueprintDiff {
    pub before_name: String,
    pub after_name: String,
    pub added_graphs: Vec<String>,
    pub removed_graphs: Vec<String>,
    pub graph_diffs: Vec<GraphDiff>,
    pub variable_changes: Vec<PropertyChange>,
}

impl BlueprintDiff {
    pub fn is_empty(&self) -> bool {
        self.added_graphs.is_empty()
            && self.removed_graphs.is_empty()
            && self.graph_diffs.iter().all(|g| g.node_diffs.is_empty())
            && self.variable_changes.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.added_graphs.len()
            + self.removed_graphs.len()
            + self
                .graph_diffs
                .iter()
                .map(|g| g.node_diffs.len())
                .sum::<usize>()
            + self.variable_changes.len()
    }
}

// ============================================================
// DIFF ENGINE
// ============================================================

/// Compare two Blueprints and return all differences
pub fn diff(before: &Blueprint, after: &Blueprint) -> BlueprintDiff {
    let mut d = BlueprintDiff {
        before_name: before.name.clone(),
        after_name: after.name.clone(),
        ..Default::default()
    };

    // Graph-level changes
    let before_graphs: HashMap<&str, &BpGraph> =
        before.graphs.iter().map(|g| (g.name.as_str(), g)).collect();
    let after_graphs: HashMap<&str, &BpGraph> =
        after.graphs.iter().map(|g| (g.name.as_str(), g)).collect();

    for name in before_graphs.keys() {
        if !after_graphs.contains_key(name) {
            d.removed_graphs.push(name.to_string());
        }
    }
    for name in after_graphs.keys() {
        if !before_graphs.contains_key(name) {
            d.added_graphs.push(name.to_string());
        }
    }

    // Diff graphs that exist in both
    for (name, before_graph) in &before_graphs {
        if let Some(after_graph) = after_graphs.get(name) {
            let graph_diff = diff_graph(before_graph, after_graph);
            if !graph_diff.node_diffs.is_empty() {
                d.graph_diffs.push(graph_diff);
            }
        }
    }

    // Variable changes
    let before_vars: HashMap<&str, &BpVariable> = before
        .variables
        .iter()
        .map(|v| (v.name.as_str(), v))
        .collect();
    let after_vars: HashMap<&str, &BpVariable> = after
        .variables
        .iter()
        .map(|v| (v.name.as_str(), v))
        .collect();

    for (name, bv) in &before_vars {
        if let Some(av) = after_vars.get(name) {
            if bv.default_value != av.default_value {
                d.variable_changes.push(PropertyChange {
                    key: name.to_string(),
                    before: bv.default_value.clone(),
                    after: av.default_value.clone(),
                });
            }
        } else {
            d.variable_changes.push(PropertyChange {
                key: name.to_string(),
                before: Some(format!("{:?}", bv.var_type.category)),
                after: None,
            });
        }
    }
    for name in after_vars.keys() {
        if !before_vars.contains_key(name) {
            d.variable_changes.push(PropertyChange {
                key: name.to_string(),
                before: None,
                after: Some(format!("{:?}", after_vars[name].var_type.category)),
            });
        }
    }

    d
}

fn diff_graph(before: &BpGraph, after: &BpGraph) -> GraphDiff {
    let mut graph_diff = GraphDiff {
        graph_name: before.name.clone(),
        node_diffs: Vec::new(),
    };

    let before_nodes: HashMap<&str, &BpNode> =
        before.nodes.iter().map(|n| (n.name.as_str(), n)).collect();
    let after_nodes: HashMap<&str, &BpNode> =
        after.nodes.iter().map(|n| (n.name.as_str(), n)).collect();

    for (name, bn) in &before_nodes {
        if let Some(an) = after_nodes.get(name) {
            if let Some(diff_kind) = diff_node(bn, an) {
                graph_diff.node_diffs.push(NodeDiff {
                    node_name: name.to_string(),
                    kind: diff_kind,
                });
            }
        } else {
            graph_diff.node_diffs.push(NodeDiff {
                node_name: name.to_string(),
                kind: NodeDiffKind::Removed((*bn).clone()),
            });
        }
    }

    for (name, an) in &after_nodes {
        if !before_nodes.contains_key(name) {
            graph_diff.node_diffs.push(NodeDiff {
                node_name: name.to_string(),
                kind: NodeDiffKind::Added((*an).clone()),
            });
        }
    }

    graph_diff
}

fn diff_node(before: &BpNode, after: &BpNode) -> Option<NodeDiffKind> {
    let mut prop_changes = Vec::new();
    let mut pin_changes = Vec::new();

    // Position change
    let pos_changed = before.pos.x != after.pos.x || before.pos.y != after.pos.y;

    // Property changes
    for (key, bval) in &before.properties {
        match after.properties.get(key) {
            Some(aval) if aval != bval => {
                prop_changes.push(PropertyChange {
                    key: key.clone(),
                    before: Some(bval.clone()),
                    after: Some(aval.clone()),
                });
            }
            None => {
                prop_changes.push(PropertyChange {
                    key: key.clone(),
                    before: Some(bval.clone()),
                    after: None,
                });
            }
            _ => { /* handled */ }
        }
    }
    for key in after.properties.keys() {
        if !before.properties.contains_key(key) {
            prop_changes.push(PropertyChange {
                key: key.clone(),
                before: None,
                after: after.properties.get(key).cloned(),
            });
        }
    }

    // Pin changes
    let before_pins: HashMap<&str, &Pin> =
        before.pins.iter().map(|p| (p.name.as_str(), p)).collect();
    let after_pins: HashMap<&str, &Pin> = after.pins.iter().map(|p| (p.name.as_str(), p)).collect();

    for (pname, bp) in &before_pins {
        if let Some(ap) = after_pins.get(pname) {
            // Default value changed?
            if bp.default_value != ap.default_value {
                pin_changes.push(PinChange {
                    pin_name: pname.to_string(),
                    kind: PinChangeKind::DefaultValueChanged {
                        before: bp.default_value.clone(),
                        after: ap.default_value.clone(),
                    },
                });
            }
            // Connection changes - track by node name
            let before_links: HashSet<&str> =
                bp.linked_to.iter().map(|l| l.node_name.as_str()).collect();
            let after_links: HashSet<&str> =
                ap.linked_to.iter().map(|l| l.node_name.as_str()).collect();
            for removed in before_links.difference(&after_links) {
                pin_changes.push(PinChange {
                    pin_name: pname.to_string(),
                    kind: PinChangeKind::ConnectionRemoved {
                        to_node: removed.to_string(),
                        to_pin: String::new(),
                    },
                });
            }
            for added in after_links.difference(&before_links) {
                pin_changes.push(PinChange {
                    pin_name: pname.to_string(),
                    kind: PinChangeKind::ConnectionAdded {
                        to_node: added.to_string(),
                        to_pin: String::new(),
                    },
                });
            }
        } else {
            pin_changes.push(PinChange {
                pin_name: pname.to_string(),
                kind: PinChangeKind::Removed,
            });
        }
    }
    for pname in after_pins.keys() {
        if !before_pins.contains_key(pname) {
            pin_changes.push(PinChange {
                pin_name: pname.to_string(),
                kind: PinChangeKind::Added,
            });
        }
    }

    if !pos_changed && prop_changes.is_empty() && pin_changes.is_empty() {
        None
    } else {
        Some(NodeDiffKind::Modified {
            position_changed: pos_changed,
            property_changes: prop_changes,
            pin_changes,
        })
    }
}

// ============================================================
// DIFF FORMATTING
// ============================================================

/// Format a BlueprintDiff as a human-readable string (like git diff but for Blueprints)
pub fn format_diff(d: &BlueprintDiff) -> String {
    if d.is_empty() {
        return format!(
            "No changes between '{}' and '{}'.",
            d.before_name, d.after_name
        );
    }

    let mut out = String::new();
    out.push_str(&format!("--- {}\n+++ {}\n", d.before_name, d.after_name));
    out.push_str(&format!("{} change(s) total\n\n", d.total_changes()));

    for graph in &d.removed_graphs {
        out.push_str(&format!("- Graph removed: {}\n", graph));
    }
    for graph in &d.added_graphs {
        out.push_str(&format!("+ Graph added: {}\n", graph));
    }

    for graph_diff in &d.graph_diffs {
        out.push_str(&format!("\n@@ Graph: {} @@\n", graph_diff.graph_name));
        for nd in &graph_diff.node_diffs {
            match &nd.kind {
                NodeDiffKind::Added(node) => {
                    out.push_str(&format!(
                        "+ Node added: {} ({})\n",
                        nd.node_name,
                        node.class.split('.').next_back().unwrap_or("")
                    ));
                }
                NodeDiffKind::Removed(node) => {
                    out.push_str(&format!(
                        "- Node removed: {} ({})\n",
                        nd.node_name,
                        node.class.split('.').next_back().unwrap_or("")
                    ));
                }
                NodeDiffKind::Modified {
                    position_changed,
                    property_changes,
                    pin_changes,
                } => {
                    out.push_str(&format!("~ Node modified: {}\n", nd.node_name));
                    if *position_changed {
                        out.push_str("    ~ position changed\n");
                    }
                    for pc in property_changes {
                        let b = pc.before.as_deref().unwrap_or("(none)");
                        let a = pc.after.as_deref().unwrap_or("(none)");
                        out.push_str(&format!("    ~ {}: {} -> {}\n", pc.key, b, a));
                    }
                    for pin_c in pin_changes {
                        match &pin_c.kind {
                            PinChangeKind::Added => {
                                out.push_str(&format!("    + pin: {}\n", pin_c.pin_name));
                            }
                            PinChangeKind::Removed => {
                                out.push_str(&format!("    - pin: {}\n", pin_c.pin_name));
                            }
                            PinChangeKind::ConnectionAdded { to_node, .. } => {
                                out.push_str(&format!(
                                    "    + connect: {} -> {}\n",
                                    pin_c.pin_name, to_node
                                ));
                            }
                            PinChangeKind::ConnectionRemoved { to_node, .. } => {
                                out.push_str(&format!(
                                    "    - connect: {} -> {}\n",
                                    pin_c.pin_name, to_node
                                ));
                            }
                            PinChangeKind::DefaultValueChanged { before, after } => {
                                out.push_str(&format!(
                                    "    ~ pin {}: default '{}' -> '{}'\n",
                                    pin_c.pin_name,
                                    before.as_deref().unwrap_or(""),
                                    after.as_deref().unwrap_or("")
                                ));
                            }
                            PinChangeKind::TypeChanged { before, after } => {
                                out.push_str(&format!(
                                    "    ~ pin {} type: {} -> {}\n",
                                    pin_c.pin_name, before, after
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    for vc in &d.variable_changes {
        let b = vc.before.as_deref().unwrap_or("(none)");
        let a = vc.after.as_deref().unwrap_or("(none)");
        if vc.before.is_none() {
            out.push_str(&format!("+ Variable added: {}: {}\n", vc.key, a));
        } else if vc.after.is_none() {
            out.push_str(&format!("- Variable removed: {}\n", vc.key));
        } else {
            out.push_str(&format!("~ Variable changed: {}: {} -> {}\n", vc.key, b, a));
        }
    }

    out
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Blueprint, BpNode, BpVariable, Pin};
    use crate::types::PinType;

    fn make_bp(name: &str) -> Blueprint {
        Blueprint::new(name, "Actor")
    }

    fn simple_node(node_name: &str, class: &str, x: i32, y: i32) -> BpNode {
        BpNode::new(class, node_name).at(x, y)
    }

    // ---- identity ----

    #[test]
    fn diff_identical_blueprints_is_empty() {
        let bp = make_bp("MyBP");
        let d = diff(&bp, &bp);
        assert!(d.is_empty(), "expected no changes for identical blueprints");
    }

    #[test]
    fn diff_identical_with_nodes_is_empty() {
        let mut bp = make_bp("MyBP");
        let node = simple_node(
            "EventBeginPlay_0",
            "/Script/BlueprintGraph.K2Node_Event",
            0,
            0,
        );
        bp.event_graph().add_node(node);
        let d = diff(&bp, &bp);
        assert!(d.is_empty());
    }

    // ---- node changes ----

    #[test]
    fn diff_detects_added_node() {
        let before = make_bp("MyBP");
        let mut after = make_bp("MyBP");
        let node = simple_node(
            "PrintString_0",
            "/Script/BlueprintGraph.K2Node_CallFunction",
            100,
            200,
        );
        after.event_graph().add_node(node);

        let d = diff(&before, &after);
        assert!(!d.is_empty());
        assert_eq!(d.graph_diffs.len(), 1);
        let nd = &d.graph_diffs[0].node_diffs[0];
        assert_eq!(nd.node_name, "PrintString_0");
        assert!(matches!(nd.kind, NodeDiffKind::Added(_)));
    }

    #[test]
    fn diff_detects_removed_node() {
        let mut before = make_bp("MyBP");
        let after = make_bp("MyBP");
        let node = simple_node(
            "PrintString_0",
            "/Script/BlueprintGraph.K2Node_CallFunction",
            100,
            200,
        );
        before.event_graph().add_node(node);

        let d = diff(&before, &after);
        assert!(!d.is_empty());
        assert_eq!(d.graph_diffs.len(), 1);
        let nd = &d.graph_diffs[0].node_diffs[0];
        assert_eq!(nd.node_name, "PrintString_0");
        assert!(matches!(nd.kind, NodeDiffKind::Removed(_)));
    }

    #[test]
    fn diff_detects_position_change() {
        let mut before = make_bp("MyBP");
        let mut after = make_bp("MyBP");
        before
            .event_graph()
            .add_node(simple_node("Node_0", "/Script/BP.K2Node_Event", 0, 0));
        after
            .event_graph()
            .add_node(simple_node("Node_0", "/Script/BP.K2Node_Event", 500, 300));

        let d = diff(&before, &after);
        assert!(!d.is_empty());
        let nd = &d.graph_diffs[0].node_diffs[0];
        match &nd.kind {
            NodeDiffKind::Modified {
                position_changed, ..
            } => assert!(*position_changed),
            _ => panic!("expected Modified"),
        }
    }

    #[test]
    fn diff_detects_property_change() {
        let mut before = make_bp("MyBP");
        let mut after = make_bp("MyBP");
        before.event_graph().add_node(
            BpNode::new("/Script/BP.K2Node_Event", "Node_0").with_property("CustomTag", "OldValue"),
        );
        after.event_graph().add_node(
            BpNode::new("/Script/BP.K2Node_Event", "Node_0").with_property("CustomTag", "NewValue"),
        );

        let d = diff(&before, &after);
        assert!(!d.is_empty());
        let nd = &d.graph_diffs[0].node_diffs[0];
        match &nd.kind {
            NodeDiffKind::Modified {
                property_changes, ..
            } => {
                assert_eq!(property_changes.len(), 1);
                assert_eq!(property_changes[0].key, "CustomTag");
                assert_eq!(property_changes[0].before.as_deref(), Some("OldValue"));
                assert_eq!(property_changes[0].after.as_deref(), Some("NewValue"));
            }
            _ => panic!("expected Modified"),
        }
    }

    // ---- connection changes ----

    #[test]
    fn diff_detects_added_connection() {
        let mut before = make_bp("MyBP");
        let mut after = make_bp("MyBP");

        let node_a = BpNode::new("/Script/BP.K2Node_Event", "EventNode")
            .with_pin(Pin::exec_output("exec_out"));
        let node_b = BpNode::new("/Script/BP.K2Node_CallFunction", "PrintNode")
            .with_pin(Pin::exec_input("exec_in"));

        before.event_graph().add_node(node_a.clone());
        before.event_graph().add_node(node_b.clone());

        after.event_graph().add_node(node_a);
        after.event_graph().add_node(node_b);
        after
            .event_graph()
            .connect("EventNode", "exec_out", "PrintNode", "exec_in");

        let d = diff(&before, &after);
        assert!(!d.is_empty());
        let graph_diff = &d.graph_diffs[0];
        let connection_added = graph_diff.node_diffs.iter().any(|nd| {
            if let NodeDiffKind::Modified { pin_changes, .. } = &nd.kind {
                pin_changes
                    .iter()
                    .any(|pc| matches!(&pc.kind, PinChangeKind::ConnectionAdded { .. }))
            } else {
                false
            }
        });
        assert!(connection_added, "expected a ConnectionAdded pin change");
    }

    #[test]
    fn diff_detects_removed_connection() {
        let mut before = make_bp("MyBP");
        let mut after = make_bp("MyBP");

        let node_a = BpNode::new("/Script/BP.K2Node_Event", "EventNode")
            .with_pin(Pin::exec_output("exec_out"));
        let node_b = BpNode::new("/Script/BP.K2Node_CallFunction", "PrintNode")
            .with_pin(Pin::exec_input("exec_in"));

        before.event_graph().add_node(node_a.clone());
        before.event_graph().add_node(node_b.clone());
        before
            .event_graph()
            .connect("EventNode", "exec_out", "PrintNode", "exec_in");

        after.event_graph().add_node(node_a);
        after.event_graph().add_node(node_b);
        // no connection in after

        let d = diff(&before, &after);
        assert!(!d.is_empty());
        let graph_diff = &d.graph_diffs[0];
        let connection_removed = graph_diff.node_diffs.iter().any(|nd| {
            if let NodeDiffKind::Modified { pin_changes, .. } = &nd.kind {
                pin_changes
                    .iter()
                    .any(|pc| matches!(&pc.kind, PinChangeKind::ConnectionRemoved { .. }))
            } else {
                false
            }
        });
        assert!(
            connection_removed,
            "expected a ConnectionRemoved pin change"
        );
    }

    // ---- variable changes ----

    #[test]
    fn diff_detects_added_variable() {
        let before = make_bp("MyBP");
        let mut after = make_bp("MyBP");
        after.add_variable(BpVariable::new("Health", PinType::float()));

        let d = diff(&before, &after);
        assert!(!d.is_empty());
        assert_eq!(d.variable_changes.len(), 1);
        assert_eq!(d.variable_changes[0].key, "Health");
        assert!(d.variable_changes[0].before.is_none());
        assert!(d.variable_changes[0].after.is_some());
    }

    #[test]
    fn diff_detects_removed_variable() {
        let mut before = make_bp("MyBP");
        let after = make_bp("MyBP");
        before.add_variable(BpVariable::new("Health", PinType::float()));

        let d = diff(&before, &after);
        assert!(!d.is_empty());
        assert_eq!(d.variable_changes.len(), 1);
        assert_eq!(d.variable_changes[0].key, "Health");
        assert!(d.variable_changes[0].before.is_some());
        assert!(d.variable_changes[0].after.is_none());
    }

    // ---- format_diff ----

    #[test]
    fn format_diff_returns_no_changes_for_empty() {
        let bp = make_bp("TestBP");
        let d = diff(&bp, &bp);
        let output = format_diff(&d);
        assert!(
            output.contains("No changes"),
            "expected 'No changes' in output, got: {}",
            output
        );
    }

    #[test]
    fn format_diff_shows_prefix_format() {
        let before = make_bp("BeforeBP");
        let mut after = make_bp("AfterBP");
        after.add_variable(BpVariable::new("Score", PinType::int()));

        let d = diff(&before, &after);
        let output = format_diff(&d);
        assert!(
            output.contains("--- BeforeBP"),
            "expected '--- BeforeBP' in output"
        );
        assert!(
            output.contains("+++ AfterBP"),
            "expected '+++ AfterBP' in output"
        );
        assert!(
            output.contains("+ Variable added:"),
            "expected variable added line"
        );
    }

    #[test]
    fn total_changes_returns_correct_count() {
        let before = make_bp("MyBP");
        let mut after = make_bp("MyBP");
        after.add_variable(BpVariable::new("Health", PinType::float()));
        after.add_variable(BpVariable::new("Mana", PinType::float()));
        after.event_graph().add_node(simple_node(
            "PrintString_0",
            "/Script/BlueprintGraph.K2Node_CallFunction",
            100,
            0,
        ));

        let d = diff(&before, &after);
        // 2 variable additions + 1 node addition = 3 total changes
        assert_eq!(d.total_changes(), 3);
    }

    #[test]
    fn diff_detects_variable_default_value_change() {
        let mut before = make_bp("MyBP");
        let mut after = make_bp("MyBP");
        before.add_variable(BpVariable::new("Health", PinType::float()).with_default("100"));
        after.add_variable(BpVariable::new("Health", PinType::float()).with_default("200"));

        let d = diff(&before, &after);
        assert!(!d.is_empty());
        assert_eq!(d.variable_changes.len(), 1);
        assert_eq!(d.variable_changes[0].before.as_deref(), Some("100"));
        assert_eq!(d.variable_changes[0].after.as_deref(), Some("200"));
    }
}
