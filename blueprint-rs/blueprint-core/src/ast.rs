use crate::types::{NodePos, PinDirection, PinType, UeGuid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type NodeProperties = HashMap<String, String>;

/// Reference to a specific pin on a specific node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinRef {
    pub node_name: String,
    pub pin_id: UeGuid,
}

/// A pin on a Blueprint node — represents an input or output connection point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    pub id: UeGuid,
    pub name: String,
    pub display_name: Option<String>,
    pub direction: PinDirection,
    pub pin_type: PinType,
    pub default_value: Option<String>,
    pub linked_to: Vec<PinRef>,
    pub is_hidden: bool,
    pub is_not_connectable: bool,
    pub is_advanced_view: bool,
    pub is_orphaned: bool,
}

impl Pin {
    pub fn new(name: impl Into<String>, direction: PinDirection, pin_type: PinType) -> Self {
        Self {
            id: UeGuid::new(),
            name: name.into(),
            display_name: None,
            direction,
            pin_type,
            default_value: None,
            linked_to: Vec::new(),
            is_hidden: false,
            is_not_connectable: false,
            is_advanced_view: false,
            is_orphaned: false,
        }
    }

    pub fn exec_input(name: impl Into<String>) -> Self {
        Self::new(name, PinDirection::Input, PinType::exec())
    }

    pub fn exec_output(name: impl Into<String>) -> Self {
        Self::new(name, PinDirection::Output, PinType::exec())
    }

    pub fn data_input(name: impl Into<String>, pt: PinType) -> Self {
        Self::new(name, PinDirection::Input, pt)
    }

    pub fn data_output(name: impl Into<String>, pt: PinType) -> Self {
        Self::new(name, PinDirection::Output, pt)
    }

    pub fn hidden(mut self) -> Self {
        self.is_hidden = true;
        self.is_not_connectable = true;
        self
    }

    pub fn with_default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }
}

/// A node in the Blueprint graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpNode {
    pub id: UeGuid,
    pub name: String,
    /// Full UE4 class path, e.g. "/Script/BlueprintGraph.K2Node_Event"
    pub class: String,
    pub pos: NodePos,
    /// Extra node-level properties serialized before pins in T3D output
    pub properties: NodeProperties,
    pub pins: Vec<Pin>,
}

impl BpNode {
    pub fn new(class: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: UeGuid::new(),
            name: name.into(),
            class: class.into(),
            pos: NodePos::default(),
            properties: NodeProperties::new(),
            pins: Vec::new(),
        }
    }

    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.pos = NodePos::new(x, y);
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    pub fn with_pin(mut self, pin: Pin) -> Self {
        self.pins.push(pin);
        self
    }

    pub fn find_pin(&self, name: &str) -> Option<&Pin> {
        self.pins.iter().find(|p| p.name == name)
    }

    pub fn find_pin_mut(&mut self, name: &str) -> Option<&mut Pin> {
        self.pins.iter_mut().find(|p| p.name == name)
    }

    pub fn pin_id(&self, name: &str) -> Option<&UeGuid> {
        self.find_pin(name).map(|p| &p.id)
    }
}

/// A Blueprint graph (EventGraph, function, macro, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpGraph {
    pub name: String,
    pub graph_type: GraphType,
    pub nodes: Vec<BpNode>,
}

/// The type of Blueprint graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphType {
    EventGraph,
    Function,
    Macro,
    Animation,
}

impl BpGraph {
    pub fn event_graph() -> Self {
        Self {
            name: "EventGraph".to_string(),
            graph_type: GraphType::EventGraph,
            nodes: Vec::new(),
        }
    }

    pub fn function(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            graph_type: GraphType::Function,
            nodes: Vec::new(),
        }
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            graph_type: GraphType::EventGraph,
            nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: BpNode) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// Connect from_pin (output) on from_node to to_pin (input) on to_node
    pub fn connect(&mut self, from_node: &str, from_pin: &str, to_node: &str, to_pin: &str) {
        // Collect pin IDs first to avoid borrow issues
        let to_pin_id = self
            .nodes
            .iter()
            .find(|n| n.name == to_node)
            .and_then(|n| n.find_pin(to_pin))
            .map(|p| p.id.clone());

        let from_pin_id = self
            .nodes
            .iter()
            .find(|n| n.name == from_node)
            .and_then(|n| n.find_pin(from_pin))
            .map(|p| p.id.clone());

        if let (Some(fid), Some(tid)) = (from_pin_id, to_pin_id) {
            // Link from output pin → to node
            if let Some(n) = self.nodes.iter_mut().find(|n| n.name == from_node) {
                if let Some(p) = n.find_pin_mut(from_pin) {
                    p.linked_to.push(PinRef {
                        node_name: to_node.to_string(),
                        pin_id: tid,
                    });
                }
            }
            // Link from input pin → from node (bidirectional)
            if let Some(n) = self.nodes.iter_mut().find(|n| n.name == to_node) {
                if let Some(p) = n.find_pin_mut(to_pin) {
                    p.linked_to.push(PinRef {
                        node_name: from_node.to_string(),
                        pin_id: fid,
                    });
                }
            }
        }
    }

    pub fn node(&self, name: &str) -> Option<&BpNode> {
        self.nodes.iter().find(|n| n.name == name)
    }

    pub fn node_mut(&mut self, name: &str) -> Option<&mut BpNode> {
        self.nodes.iter_mut().find(|n| n.name == name)
    }
}

/// Blueprint variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpVariable {
    pub name: String,
    pub var_type: PinType,
    pub default_value: Option<String>,
    pub is_exposed: bool,
    pub category: Option<String>,
    pub tooltip: Option<String>,
    pub replication: ReplicationMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReplicationMode {
    None,
    Replicated,
    RepNotify,
}

impl BpVariable {
    pub fn new(name: impl Into<String>, var_type: PinType) -> Self {
        Self {
            name: name.into(),
            var_type,
            default_value: None,
            is_exposed: false,
            category: None,
            tooltip: None,
            replication: ReplicationMode::None,
        }
    }

    pub fn exposed(mut self) -> Self {
        self.is_exposed = true;
        self
    }

    pub fn in_category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    pub fn with_default(mut self, val: impl Into<String>) -> Self {
        self.default_value = Some(val.into());
        self
    }

    pub fn replicated(mut self) -> Self {
        self.replication = ReplicationMode::Replicated;
        self
    }
}

/// A complete Unreal Engine 4 Blueprint asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub name: String,
    pub parent_class: String,
    pub graphs: Vec<BpGraph>,
    pub variables: Vec<BpVariable>,
    pub interfaces: Vec<String>,
}

impl Blueprint {
    pub fn new(name: impl Into<String>, parent_class: impl Into<String>) -> Self {
        let mut bp = Self {
            name: name.into(),
            parent_class: parent_class.into(),
            graphs: Vec::new(),
            variables: Vec::new(),
            interfaces: Vec::new(),
        };
        bp.graphs.push(BpGraph::event_graph());
        bp
    }

    pub fn event_graph(&mut self) -> &mut BpGraph {
        if !self.graphs.iter().any(|g| g.name == "EventGraph") {
            self.graphs.push(BpGraph::event_graph());
        }
        self.graphs
            .iter_mut()
            .find(|g| g.name == "EventGraph")
            .unwrap()
    }

    pub fn function_graph(&mut self, name: impl Into<String>) -> &mut BpGraph {
        let name = name.into();
        if !self.graphs.iter().any(|g| g.name == name) {
            self.graphs.push(BpGraph::function(name.clone()));
        }
        self.graphs.iter_mut().find(|g| g.name == name).unwrap()
    }

    pub fn add_variable(&mut self, var: BpVariable) {
        self.variables.push(var);
    }

    pub fn implement_interface(&mut self, interface: impl Into<String>) {
        self.interfaces.push(interface.into());
    }

    pub fn all_nodes(&self) -> impl Iterator<Item = &BpNode> {
        self.graphs.iter().flat_map(|g| g.nodes.iter())
    }
}
