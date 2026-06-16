//! RdfToBlueprintCodegen — generate Blueprints from declarative specs.

use blueprint_core::{Blueprint, BlueprintBuilder, VarType, T3dSerializer, JsonSerializer};

/// Spec for a Blueprint variable.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VarSpec {
    pub name: String,
    pub var_type: String,
    pub default: String,
}

/// A simple declarative spec for generating a Blueprint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlueprintSpec {
    pub name: String,
    pub parent_class: String,
    /// Event names to add — recognized: "begin_play", "end_play", "tick".
    pub events: Vec<String>,
    pub variables: Vec<VarSpec>,
    /// If set, add a PrintString node to BeginPlay with this message.
    pub print_on_begin: Option<String>,
}

/// Stateless Blueprint code-generator.
pub struct BlueprintCodegen;

impl BlueprintCodegen {
    /// Generate a [`Blueprint`] from a declarative [`BlueprintSpec`].
    pub fn from_spec(spec: &BlueprintSpec) -> Blueprint {
        let mut builder = BlueprintBuilder::new(&spec.name, &spec.parent_class);

        // Add variables
        for var in &spec.variables {
            let vt = parse_var_type(&var.var_type);
            builder.add_variable_mut(
                &var.name,
                vt,
                if var.default.is_empty() { None } else { Some(var.default.clone()) },
            );
        }

        // Add event nodes
        for event in &spec.events {
            match event.as_str() {
                "begin_play" => {
                    let ev = builder.begin_play_node();
                    if let Some(msg) = &spec.print_on_begin {
                        let ps = builder.print_string(msg.clone());
                        builder.exec_connect(&ev, &ps);
                    }
                }
                "end_play" => { builder.end_play_node(); }
                "tick" => { builder.tick_node(); }
                other => { builder.custom_event_node(other); }
            }
        }

        builder.build()
    }

    /// Generate a Blueprint from a spec and serialize it to UE4 T3D format.
    pub fn to_t3d(spec: &BlueprintSpec) -> String {
        let bp = Self::from_spec(spec);
        T3dSerializer::serialize(&bp)
    }

    /// Generate a Blueprint from a spec and serialize it to pretty JSON.
    pub fn to_json(spec: &BlueprintSpec) -> Result<String, String> {
        let bp = Self::from_spec(spec);
        JsonSerializer::serialize(&bp).map_err(|e| e.to_string())
    }
}

fn parse_var_type(s: &str) -> VarType {
    match s.to_lowercase().as_str() {
        "int" | "integer" => VarType::Int,
        "float" | "f32" => VarType::Float,
        "bool" | "boolean" => VarType::Bool,
        "string" | "str" => VarType::String,
        "name" => VarType::Name,
        _ => VarType::String,
    }
}
