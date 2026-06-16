//! T3D serializer — converts a Blueprint AST to UE4 T3D copy/paste format.
//!
//! The output can be pasted directly into the UE4 Blueprint editor Event Graph
//! (Ctrl+V) and the nodes will appear with their connections intact.

use crate::ast::{Blueprint, BpNode, Pin};
use crate::types::{PinDirection, PinCategory};

pub struct T3dSerializer;

impl T3dSerializer {
    /// Serialize a `Blueprint` to UE4 T3D format string.
    pub fn serialize(bp: &Blueprint) -> String {
        let mut out = String::new();

        out.push_str("Begin Object Class=/Script/BlueprintGraph.K2Node_Tunnel Name=\"Graph\"\n");

        for graph in &bp.graphs {
            out.push_str(&format!("   // Graph: {}\n", graph.name));
            for node in &graph.nodes {
                out.push_str(&serialize_node(node));
                out.push('\n');
            }
        }

        out.push_str("End Object\n");
        out
    }
}

fn serialize_node(node: &BpNode) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "Begin Object Class={} Name=\"{}\"\n",
        node.class, node.name
    ));

    // Node position
    out.push_str(&format!(
        "   NodePosX={}\n   NodePosY={}\n",
        node.pos.x, node.pos.y
    ));

    // Node GUID
    out.push_str(&format!("   NodeGuid={}\n", node.id));

    // Extra properties (e.g. EventReference, FunctionReference)
    // Sort for deterministic output
    let mut props: Vec<(&String, &String)> = node.properties.iter().collect();
    props.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in &props {
        out.push_str(&format!("   {}={}\n", k, v));
    }

    // Pins
    for (i, pin) in node.pins.iter().enumerate() {
        out.push_str(&serialize_pin(pin, i));
    }

    out.push_str("End Object\n");
    out
}

fn serialize_pin(pin: &Pin, index: usize) -> String {
    let mut parts = Vec::new();

    parts.push(format!("PinId={}", pin.id));
    parts.push(format!("PinName=\"{}\"", pin.name));

    if let Some(dn) = &pin.display_name {
        parts.push(format!("PinFriendlyName=\"{}\"", dn));
    }

    // Direction
    let dir_str = match pin.direction {
        PinDirection::Input => "EGPD_Input",
        PinDirection::Output => "EGPD_Output",
    };
    parts.push(format!("Direction=\"{}\"", dir_str));

    // PinType — PinCategory
    parts.push(format!("PinType.PinCategory=\"{}\"", pin.pin_type.category.as_str()));

    if let Some(sub) = &pin.pin_type.sub_category {
        parts.push(format!("PinType.PinSubCategory=\"{}\"", sub));
    }

    if let Some(obj) = &pin.pin_type.sub_category_object {
        parts.push(format!("PinType.PinSubCategoryObject={}", obj));
    }

    // LinkedTo list
    for link in &pin.linked_to {
        parts.push(format!(
            "LinkedTo=({}({}))",
            link.node_name, link.pin_id
        ));
    }

    // Default value (skip for exec pins)
    if pin.pin_type.category != PinCategory::Exec {
        if let Some(dv) = &pin.default_value {
            parts.push(format!("DefaultValue=\"{}\"", dv));
        }
    }

    if pin.is_hidden {
        parts.push("bHidden=True".to_string());
    }
    if pin.is_not_connectable {
        parts.push("bNotConnectable=True".to_string());
    }
    if pin.is_advanced_view {
        parts.push("bAdvancedView=True".to_string());
    }
    if pin.is_orphaned {
        parts.push("bOrphanedPin=True".to_string());
    }

    format!(
        "   CustomProperties Pin ({})\n",
        parts.join(",")
    )
}
