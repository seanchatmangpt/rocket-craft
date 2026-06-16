use crate::ast::*;
use crate::types::*;
use std::collections::HashSet;

// ============================================================
// MERMAID DIAGRAM
// ============================================================

/// Render a Blueprint as a Mermaid.js flowchart (LR = left-to-right)
/// Output can be pasted into https://mermaid.live or GitHub markdown
pub fn render_mermaid(blueprint: &Blueprint) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "---\ntitle: Blueprint — {} ({})\n---\n",
        blueprint.name, blueprint.parent_class
    ));

    for graph in &blueprint.graphs {
        out.push_str(&format!("%%  Graph: {}\n", graph.name));
        out.push_str("flowchart LR\n");

        // Node definitions with shape based on class
        for node in &graph.nodes {
            let short_class = node.class.split('.').last().unwrap_or(&node.class);
            let safe_name = sanitize_mermaid_id(&node.name);
            let label = mermaid_label(node);

            // Different shapes for different node types
            let shape = if short_class.contains("Event") || short_class.contains("CustomEvent") {
                format!("{}([{}])", safe_name, label) // stadium shape for events
            } else if short_class.contains("IfThenElse") {
                format!("{}{{{{{}}}}}", safe_name, label) // diamond for branch
            } else if short_class.contains("ForLoop") || short_class.contains("WhileLoop") {
                format!("{}[/​{}/​]", safe_name, label) // parallelogram for loops
            } else {
                format!("{}[{}]", safe_name, label) // default rectangle
            };
            out.push_str(&format!("    {}\n", shape));
        }

        // Connections (exec flow only for clarity)
        let mut drawn: HashSet<String> = HashSet::new();
        for node in &graph.nodes {
            for pin in &node.pins {
                if pin.direction == PinDirection::Output
                    && pin.pin_type.category == PinCategory::Exec
                {
                    for link in &pin.linked_to {
                        let edge_key = format!("{}-->{}", node.name, link.node_name);
                        if drawn.insert(edge_key) {
                            let edge_label = if pin.name != "then" && pin.name != "OutputDelegate" {
                                format!(" -->|{}| ", pin.name)
                            } else {
                                " --> ".to_string()
                            };
                            out.push_str(&format!(
                                "    {}{}{}\n",
                                sanitize_mermaid_id(&node.name),
                                edge_label,
                                sanitize_mermaid_id(&link.node_name)
                            ));
                        }
                    }
                }
            }
        }

        // Add style classes
        out.push_str("    classDef event fill:#90EE90,stroke:#228B22,color:#000\n");
        out.push_str("    classDef branch fill:#FFD700,stroke:#B8860B,color:#000\n");
        out.push_str("    classDef loop fill:#87CEEB,stroke:#4682B4,color:#000\n");
        out.push_str("    classDef default fill:#F5F5F5,stroke:#808080,color:#000\n");

        // Apply styles
        for node in &graph.nodes {
            let short_class = node.class.split('.').last().unwrap_or(&node.class);
            let safe_name = sanitize_mermaid_id(&node.name);
            let class = if short_class.contains("Event") || short_class.contains("CustomEvent") {
                "event"
            } else if short_class.contains("IfThenElse") {
                "branch"
            } else if short_class.contains("Loop") {
                "loop"
            } else {
                "default"
            };
            out.push_str(&format!("    class {} {}\n", safe_name, class));
        }
    }

    out
}

fn sanitize_mermaid_id(s: &str) -> String {
    s.replace('-', "_").replace(' ', "_").replace('.', "_")
}

fn mermaid_label(node: &BpNode) -> String {
    let short_class = node.class.split('.').last().unwrap_or(&node.class);
    let short_class = short_class.replace("K2Node_", "");
    // Shorten name if it's the generic counter name
    if node.name.starts_with("K2Node_") {
        short_class.to_string()
    } else {
        node.name.clone()
    }
}

// ============================================================
// GRAPHVIZ DOT FORMAT
// ============================================================

/// Render a Blueprint as a GraphViz DOT file
/// Run: dot -Tsvg output.dot -o graph.svg
pub fn render_dot(blueprint: &Blueprint) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "// Blueprint: {} ({})\n",
        blueprint.name, blueprint.parent_class
    ));
    out.push_str("digraph G {\n");
    out.push_str("    rankdir=LR;\n");
    out.push_str("    node [shape=box, style=filled, fontname=\"Helvetica\"];\n");
    out.push_str("    edge [fontname=\"Helvetica\", fontsize=10];\n\n");

    for (graph_idx, graph) in blueprint.graphs.iter().enumerate() {
        out.push_str(&format!("    subgraph cluster_{} {{\n", graph_idx));
        out.push_str(&format!("        label=\"{}\";\n", graph.name));
        out.push_str("        style=dashed;\n");

        for node in &graph.nodes {
            let short_class = node.class.split('.').last().unwrap_or(&node.class);
            let label = format!(
                "{}\n[{}]",
                node.name,
                short_class.replace("K2Node_", "")
            );
            let color = dot_color_for_node(node);
            let safe_id = format!("n_{}_{}", graph_idx, sanitize_dot_id(&node.name));
            out.push_str(&format!(
                "        {} [label=\"{}\", fillcolor=\"{}\"];\n",
                safe_id, label, color
            ));
        }

        // Exec edges
        for node in &graph.nodes {
            let from_id = format!("n_{}_{}", graph_idx, sanitize_dot_id(&node.name));
            for pin in &node.pins {
                if pin.direction == PinDirection::Output
                    && pin.pin_type.category == PinCategory::Exec
                {
                    for link in &pin.linked_to {
                        let to_id = format!(
                            "n_{}_{}",
                            graph_idx,
                            sanitize_dot_id(&link.node_name)
                        );
                        let edge_label = if pin.name != "then" { &pin.name } else { "" };
                        out.push_str(&format!(
                            "        {} -> {} [label=\"{}\", color=\"#333\"];\n",
                            from_id, to_id, edge_label
                        ));
                    }
                }
            }
        }

        out.push_str("    }\n");
    }

    out.push_str("}\n");
    out
}

fn sanitize_dot_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

fn dot_color_for_node(node: &BpNode) -> &'static str {
    let class = node.class.split('.').last().unwrap_or("");
    if class.contains("Event") || class.contains("CustomEvent") {
        "#90EE90"
    } else if class.contains("IfThenElse") {
        "#FFD700"
    } else if class.contains("Loop") || class.contains("While") {
        "#87CEEB"
    } else if class.contains("CallFunction") {
        "#F5DEB3"
    } else {
        "#F5F5F5"
    }
}

// ============================================================
// ASCII ART
// ============================================================

/// Render a Blueprint graph as ASCII art in the terminal
pub fn render_ascii(blueprint: &Blueprint) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "╔══ Blueprint: {} (parent: {}) ══╗\n",
        blueprint.name, blueprint.parent_class
    ));

    for graph in &blueprint.graphs {
        out.push_str(&format!(
            "\n┌─ Graph: {} ─────────────────────────────────\n",
            graph.name
        ));

        if graph.nodes.is_empty() {
            out.push_str("│  (empty graph)\n");
        } else {
            // Find exec chains starting from nodes with no exec predecessors
            let exec_chains = find_exec_chains(graph);
            let mut printed: HashSet<&str> = HashSet::new();

            for chain in &exec_chains {
                let mut parts = Vec::new();
                for node_name in chain {
                    if let Some(node) = graph.nodes.iter().find(|n| n.name == *node_name) {
                        parts.push(ascii_node_box(node));
                        printed.insert(node_name.as_str());
                    }
                }
                if !parts.is_empty() {
                    out.push_str("│  ");
                    out.push_str(&parts.join(" ──▶ "));
                    out.push('\n');
                }
            }

            // Print unconnected nodes
            for node in &graph.nodes {
                if !printed.contains(node.name.as_str()) {
                    out.push_str(&format!("│  {}\n", ascii_node_box(node)));
                }
            }
        }

        out.push_str("└─────────────────────────────────────────────\n");
    }

    // Variables section
    if !blueprint.variables.is_empty() {
        out.push_str("\n┌─ Variables ─────────────────────────────────\n");
        for var in &blueprint.variables {
            let default = var.default_value.as_deref().unwrap_or("(none)");
            out.push_str(&format!(
                "│  {} : {:?} = {}\n",
                var.name, var.var_type.category, default
            ));
        }
        out.push_str("└─────────────────────────────────────────────\n");
    }

    out
}

fn ascii_node_box(node: &BpNode) -> String {
    let short_class = node.class.split('.').last().unwrap_or(&node.class);
    let short_class = short_class.replace("K2Node_", "");
    format!("[{} ({})]", node.name, short_class)
}

fn find_exec_chains(graph: &BpGraph) -> Vec<Vec<String>> {
    let has_predecessors: HashSet<&str> = graph
        .nodes
        .iter()
        .flat_map(|n| {
            n.pins
                .iter()
                .filter(|p| {
                    p.direction == PinDirection::Output
                        && p.pin_type.category == PinCategory::Exec
                })
                .flat_map(|p| p.linked_to.iter().map(|l| l.node_name.as_str()))
        })
        .collect();

    let mut chains = Vec::new();
    for node in &graph.nodes {
        if !has_predecessors.contains(node.name.as_str()) {
            let mut chain: Vec<String> = Vec::new();
            let mut current: Option<&BpNode> = Some(node);
            let mut visited: HashSet<&str> = HashSet::new();

            while let Some(cur_node) = current {
                if !visited.insert(cur_node.name.as_str()) {
                    break;
                }
                chain.push(cur_node.name.clone());

                // Follow the first exec output link
                current = cur_node
                    .pins
                    .iter()
                    .filter(|p| {
                        p.direction == PinDirection::Output
                            && p.pin_type.category == PinCategory::Exec
                            && !p.linked_to.is_empty()
                    })
                    .flat_map(|p| p.linked_to.iter())
                    .next()
                    .and_then(|link| graph.nodes.iter().find(|n| n.name == link.node_name));
            }

            if !chain.is_empty() {
                chains.push(chain);
            }
        }
    }

    chains
}

// ============================================================
// SUMMARY / STATS
// ============================================================

/// Generate a compact text summary of a Blueprint
pub fn render_summary(blueprint: &Blueprint) -> String {
    let total_nodes: usize = blueprint.graphs.iter().map(|g| g.nodes.len()).sum();
    let total_connections: usize = blueprint
        .graphs
        .iter()
        .flat_map(|g| g.nodes.iter())
        .flat_map(|n| n.pins.iter())
        .filter(|p| p.direction == PinDirection::Output)
        .map(|p| p.linked_to.len())
        .sum();

    let mut out = format!(
        "Blueprint: {} ({})\n",
        blueprint.name, blueprint.parent_class
    );
    out.push_str(&format!("  Graphs:      {}\n", blueprint.graphs.len()));
    out.push_str(&format!("  Total nodes: {}\n", total_nodes));
    out.push_str(&format!("  Connections: {}\n", total_connections));
    out.push_str(&format!("  Variables:   {}\n", blueprint.variables.len()));

    for graph in &blueprint.graphs {
        let event_count = graph
            .nodes
            .iter()
            .filter(|n| {
                n.class.contains("K2Node_Event") || n.class.contains("K2Node_CustomEvent")
            })
            .count();
        out.push_str(&format!(
            "  Graph '{}': {} nodes, {} event(s)\n",
            graph.name,
            graph.nodes.len(),
            event_count
        ));
    }

    out
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BpNode, Blueprint};
    #[allow(unused_imports)]
    use crate::types::PinType;

    fn make_blueprint_empty() -> Blueprint {
        Blueprint::new("TestBP", "Actor")
    }

    fn make_event_node(name: &str) -> BpNode {
        BpNode::new("/Script/BlueprintGraph.K2Node_Event", name)
    }

    fn make_call_node(name: &str) -> BpNode {
        BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", name)
    }

    fn make_blueprint_with_two_connected_nodes() -> Blueprint {
        let mut bp = Blueprint::new("TestBP", "Actor");
        let mut event = make_event_node("K2Node_Event_0");
        let exec_in_pin = Pin::exec_input("execute");

        let mut exec_out = Pin::exec_output("then");
        exec_out.linked_to.push(PinRef {
            node_name: "K2Node_CallFunction_0".to_string(),
            pin_id: exec_in_pin.id.clone(),
        });

        event = event.with_pin(exec_out);
        let call = make_call_node("K2Node_CallFunction_0").with_pin(exec_in_pin);

        let graph = bp.event_graph();
        graph.add_node(event);
        graph.add_node(call);
        bp
    }

    // --- render_mermaid tests ---

    #[test]
    fn mermaid_empty_blueprint_starts_with_frontmatter() {
        let bp = make_blueprint_empty();
        let out = render_mermaid(&bp);
        assert!(out.starts_with("---"), "Expected frontmatter start, got: {}", out);
    }

    #[test]
    fn mermaid_empty_blueprint_contains_flowchart_lr() {
        let bp = make_blueprint_empty();
        let out = render_mermaid(&bp);
        assert!(out.contains("flowchart LR"), "Expected 'flowchart LR' in output");
    }

    #[test]
    fn mermaid_connected_nodes_produces_arrow() {
        let bp = make_blueprint_with_two_connected_nodes();
        let out = render_mermaid(&bp);
        assert!(out.contains("-->"), "Expected '-->' edge in mermaid output, got:\n{}", out);
    }

    #[test]
    fn mermaid_event_node_uses_stadium_shape() {
        let mut bp = Blueprint::new("TestBP", "Actor");
        let event = make_event_node("MyEvent");
        bp.event_graph().add_node(event);
        let out = render_mermaid(&bp);
        assert!(out.contains("(["), "Expected stadium shape '([' for event node, got:\n{}", out);
    }

    #[test]
    fn mermaid_event_node_gets_event_class() {
        let mut bp = Blueprint::new("TestBP", "Actor");
        let event = make_event_node("MyEvent");
        bp.event_graph().add_node(event);
        let out = render_mermaid(&bp);
        assert!(out.contains("class MyEvent event"), "Expected 'class MyEvent event' in output:\n{}", out);
    }

    #[test]
    fn mermaid_regular_node_gets_default_class() {
        let mut bp = Blueprint::new("TestBP", "Actor");
        let call = make_call_node("MyCall");
        bp.event_graph().add_node(call);
        let out = render_mermaid(&bp);
        assert!(out.contains("class MyCall default"), "Expected 'class MyCall default' in output:\n{}", out);
    }

    // --- render_dot tests ---

    #[test]
    fn dot_output_contains_digraph() {
        let bp = make_blueprint_empty();
        let out = render_dot(&bp);
        assert!(out.contains("digraph G {"), "Expected 'digraph G {{' in DOT output");
    }

    #[test]
    fn dot_nodes_have_fillcolor() {
        let mut bp = Blueprint::new("TestBP", "Actor");
        let event = make_event_node("MyEvent");
        bp.event_graph().add_node(event);
        let out = render_dot(&bp);
        assert!(out.contains("fillcolor="), "Expected 'fillcolor=' in DOT output:\n{}", out);
    }

    #[test]
    fn dot_event_node_gets_green_color() {
        let mut bp = Blueprint::new("TestBP", "Actor");
        let event = make_event_node("MyEvent");
        bp.event_graph().add_node(event);
        let out = render_dot(&bp);
        assert!(out.contains("#90EE90"), "Expected green fillcolor for event node:\n{}", out);
    }

    // --- render_ascii tests ---

    #[test]
    fn ascii_single_node_contains_node_name() {
        let mut bp = Blueprint::new("TestBP", "Actor");
        let node = make_call_node("MyFunction");
        bp.event_graph().add_node(node);
        let out = render_ascii(&bp);
        assert!(out.contains("MyFunction"), "Expected node name in ASCII output:\n{}", out);
    }

    #[test]
    fn ascii_exec_chain_shows_arrow() {
        let bp = make_blueprint_with_two_connected_nodes();
        let out = render_ascii(&bp);
        assert!(out.contains("──▶"), "Expected chain arrow '──▶' in ASCII output:\n{}", out);
    }

    #[test]
    fn ascii_empty_graph_says_empty() {
        let bp = make_blueprint_empty();
        let out = render_ascii(&bp);
        assert!(out.contains("empty graph"), "Expected '(empty graph)' in ASCII output:\n{}", out);
    }

    // --- render_summary tests ---

    #[test]
    fn summary_correct_node_count() {
        let mut bp = Blueprint::new("TestBP", "Actor");
        bp.event_graph().add_node(make_event_node("Ev1"));
        bp.event_graph().add_node(make_call_node("Call1"));
        let out = render_summary(&bp);
        assert!(out.contains("Total nodes: 2"), "Expected 'Total nodes: 2' in summary:\n{}", out);
    }

    #[test]
    fn summary_correct_graph_count() {
        let bp = make_blueprint_empty();
        let out = render_summary(&bp);
        assert!(out.contains("Graphs:      1"), "Expected 'Graphs:      1' in summary:\n{}", out);
    }

    #[test]
    fn summary_contains_blueprint_name() {
        let bp = make_blueprint_empty();
        let out = render_summary(&bp);
        assert!(out.contains("TestBP"), "Expected blueprint name in summary:\n{}", out);
    }

    // --- sanitize_mermaid_id tests ---

    #[test]
    fn sanitize_replaces_dash_with_underscore() {
        assert_eq!(sanitize_mermaid_id("my-node"), "my_node");
    }

    #[test]
    fn sanitize_replaces_space_with_underscore() {
        assert_eq!(sanitize_mermaid_id("my node"), "my_node");
    }

    #[test]
    fn sanitize_replaces_dot_with_underscore() {
        assert_eq!(sanitize_mermaid_id("my.node"), "my_node");
    }

    #[test]
    fn sanitize_leaves_valid_ids_unchanged() {
        assert_eq!(sanitize_mermaid_id("myNode_123"), "myNode_123");
    }
}
