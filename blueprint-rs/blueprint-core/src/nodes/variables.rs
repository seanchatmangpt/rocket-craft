use crate::ast::{BpNode, Pin};
use crate::types::PinType;

// ======= VARIABLE GET/SET =======

/// Get the value of a Blueprint variable (self-context).
/// Creates a getter node with one output pin of the given type.
pub fn get_variable(
    node_name: impl Into<String>,
    var_name: impl Into<String>,
    var_type: PinType,
) -> BpNode {
    let vname = var_name.into();
    let output_pin = Pin::data_output(&vname, var_type);
    BpNode::new("/Script/BlueprintGraph.K2Node_VariableGet", node_name)
        .with_property(
            "VariableReference",
            format!("(MemberName=\"{}\",bSelfContext=True)", vname),
        )
        .with_pin(output_pin)
}

/// Set the value of a Blueprint variable (self-context).
/// Creates a setter node with exec pins and input/output data pins.
pub fn set_variable(
    node_name: impl Into<String>,
    var_name: impl Into<String>,
    var_type: PinType,
) -> BpNode {
    let vname = var_name.into();
    let input_pin = Pin::data_input(&vname, var_type.clone());
    let output_pin = Pin::data_output(&vname, var_type);
    BpNode::new("/Script/BlueprintGraph.K2Node_VariableSet", node_name)
        .with_property(
            "VariableReference",
            format!("(MemberName=\"{}\",bSelfContext=True)", vname),
        )
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(input_pin)
        .with_pin(output_pin)
}

/// Get variable from a specific object (not self-context).
pub fn get_variable_from_object(
    node_name: impl Into<String>,
    var_name: impl Into<String>,
    var_type: PinType,
    object_class: impl Into<String>,
) -> BpNode {
    let vname = var_name.into();
    let class = object_class.into();
    let output_pin = Pin::data_output(&vname, var_type.clone());
    BpNode::new("/Script/BlueprintGraph.K2Node_VariableGet", node_name)
        .with_property(
            "VariableReference",
            format!(
                "(MemberParent=Class'{}',MemberName=\"{}\",bSelfContext=False)",
                class, vname
            ),
        )
        .with_pin(Pin::data_input("self", PinType::object(class)))
        .with_pin(output_pin)
}

/// Set variable on a specific object (not self-context).
pub fn set_variable_on_object(
    node_name: impl Into<String>,
    var_name: impl Into<String>,
    var_type: PinType,
    object_class: impl Into<String>,
) -> BpNode {
    let vname = var_name.into();
    let class = object_class.into();
    let input_pin = Pin::data_input(&vname, var_type.clone());
    let output_pin = Pin::data_output(&vname, var_type);
    BpNode::new("/Script/BlueprintGraph.K2Node_VariableSet", node_name)
        .with_property(
            "VariableReference",
            format!(
                "(MemberParent=Class'{}',MemberName=\"{}\",bSelfContext=False)",
                class, vname
            ),
        )
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(Pin::data_input("self", PinType::object(class)))
        .with_pin(input_pin)
        .with_pin(output_pin)
}

// ======= ARRAY OPERATIONS =======

const ARRAY_LIB: &str = "Class'/Script/Engine.KismetArrayLibrary'";

fn array_call_node(node_name: impl Into<String>, function_name: &str) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", node_name).with_property(
        "FunctionReference",
        format!(
            "(MemberParent={},MemberName=\"{}\")",
            ARRAY_LIB, function_name
        ),
    )
}

/// Get array item by index.
pub fn array_get(node_name: impl Into<String>, item_type: PinType) -> BpNode {
    array_call_node(node_name, "Array_Get")
        .with_pin(Pin::data_input("TargetArray", item_type.clone().as_array()))
        .with_pin(Pin::data_input("Index", PinType::int()))
        .with_pin(Pin::data_output("ReturnValue", item_type))
}

/// Set array item at index.
pub fn array_set(node_name: impl Into<String>, item_type: PinType) -> BpNode {
    array_call_node(node_name, "Array_Set")
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(Pin::data_input("TargetArray", item_type.clone().as_array()))
        .with_pin(Pin::data_input("Index", PinType::int()))
        .with_pin(Pin::data_input("Item", item_type))
}

/// Get array length.
pub fn array_length(node_name: impl Into<String>) -> BpNode {
    array_call_node(node_name, "Array_Length")
        .with_pin(Pin::data_input(
            "TargetArray",
            PinType::wildcard().as_array(),
        ))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Add item to array.
pub fn array_add(node_name: impl Into<String>, item_type: PinType) -> BpNode {
    array_call_node(node_name, "Array_Add")
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(Pin::data_input("TargetArray", item_type.clone().as_array()))
        .with_pin(Pin::data_input("NewItem", item_type))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Remove item from array at index.
pub fn array_remove(node_name: impl Into<String>) -> BpNode {
    array_call_node(node_name, "Array_Remove")
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(Pin::data_input(
            "TargetArray",
            PinType::wildcard().as_array(),
        ))
        .with_pin(Pin::data_input("Index", PinType::int()))
}

/// Clear an array.
pub fn array_clear(node_name: impl Into<String>) -> BpNode {
    array_call_node(node_name, "Array_Clear")
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(Pin::data_input(
            "TargetArray",
            PinType::wildcard().as_array(),
        ))
}

/// Check if array contains an item.
pub fn array_contains(node_name: impl Into<String>, item_type: PinType) -> BpNode {
    array_call_node(node_name, "Array_Contains")
        .with_pin(Pin::data_input("TargetArray", item_type.clone().as_array()))
        .with_pin(Pin::data_input("ItemToFind", item_type))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

/// Find item index in array.
pub fn array_find(node_name: impl Into<String>, item_type: PinType) -> BpNode {
    array_call_node(node_name, "Array_Find")
        .with_pin(Pin::data_input("TargetArray", item_type.clone().as_array()))
        .with_pin(Pin::data_input("ItemToFind", item_type))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

// ======= CAST NODES =======

/// Cast an object to a specific class.
/// Creates a K2Node_DynamicCast node with Object input, Cast Success/Failed exec outputs,
/// and a typed output pin for the cast result.
pub fn cast_to(
    node_name: impl Into<String>,
    target_class: impl Into<String>,
    target_class_path: impl Into<String>,
) -> BpNode {
    let class_name = target_class.into();
    let class_path = target_class_path.into();
    let cast_output_name = format!("As {}", class_name);
    BpNode::new("/Script/BlueprintGraph.K2Node_DynamicCast", node_name)
        .with_property("TargetType", format!("Class'{}'", class_path))
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::data_input("Object", PinType::object("")))
        .with_pin(Pin::exec_output("Cast Success"))
        .with_pin(Pin::exec_output("Cast Failed"))
        .with_pin(Pin::data_output(
            cast_output_name,
            PinType::object(class_path),
        ))
}

// ======= LITERAL VALUE NODES =======

/// Create a literal integer node.
pub fn literal_int(node_name: impl Into<String>, value: i32) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_Literal", node_name)
        .with_pin(Pin::data_output("Value", PinType::int()).with_default(value.to_string()))
}

/// Create a literal float node.
pub fn literal_float(node_name: impl Into<String>, value: f32) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_Literal", node_name)
        .with_pin(Pin::data_output("Value", PinType::float()).with_default(value.to_string()))
}

/// Create a literal string node.
pub fn literal_string(node_name: impl Into<String>, value: impl Into<String>) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_Literal", node_name)
        .with_pin(Pin::data_output("Value", PinType::string()).with_default(value.into()))
}

/// Create a literal bool node.
pub fn literal_bool(node_name: impl Into<String>, value: bool) -> BpNode {
    let default = if value { "true" } else { "false" };
    BpNode::new("/Script/BlueprintGraph.K2Node_Literal", node_name)
        .with_pin(Pin::data_output("Value", PinType::bool()).with_default(default))
}

/// Create a literal vector node (FVector) using KismetMathLibrary's MakeVector.
pub fn literal_vector(node_name: impl Into<String>, x: f32, y: f32, z: f32) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_CallFunction", node_name)
        .with_property(
            "FunctionReference",
            "(MemberParent=Class'/Script/Engine.KismetMathLibrary',MemberName=\"MakeVector\")",
        )
        .with_pin(Pin::data_input("X", PinType::float()).with_default(x.to_string()))
        .with_pin(Pin::data_input("Y", PinType::float()).with_default(y.to_string()))
        .with_pin(Pin::data_input("Z", PinType::float()).with_default(z.to_string()))
        .with_pin(Pin::data_output(
            "ReturnValue",
            PinType::struct_type("/Script/CoreUObject.Vector"),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContainerType, PinCategory, PinDirection};

    #[test]
    fn get_variable_creates_correct_variable_reference_property() {
        let node = get_variable("GetHP", "Health", PinType::float());
        assert_eq!(node.class, "/Script/BlueprintGraph.K2Node_VariableGet");
        let prop = node
            .properties
            .get("VariableReference")
            .expect("VariableReference property missing");
        assert_eq!(prop, "(MemberName=\"Health\",bSelfContext=True)");
        assert_eq!(node.pins.len(), 1);
        let pin = &node.pins[0];
        assert_eq!(pin.name, "Health");
        assert_eq!(pin.direction, PinDirection::Output);
    }

    #[test]
    fn set_variable_has_execute_then_exec_pins_plus_data_input_and_output() {
        let node = set_variable("SetHP", "Health", PinType::float());
        assert_eq!(node.class, "/Script/BlueprintGraph.K2Node_VariableSet");
        let prop = node
            .properties
            .get("VariableReference")
            .expect("VariableReference property missing");
        assert_eq!(prop, "(MemberName=\"Health\",bSelfContext=True)");
        assert_eq!(node.pins.len(), 4);

        let exec_in = node.find_pin("execute").expect("execute pin missing");
        assert_eq!(exec_in.direction, PinDirection::Input);
        assert_eq!(exec_in.pin_type.category, PinCategory::Exec);

        let exec_out = node.find_pin("then").expect("then pin missing");
        assert_eq!(exec_out.direction, PinDirection::Output);
        assert_eq!(exec_out.pin_type.category, PinCategory::Exec);

        let data_in = node
            .pins
            .iter()
            .find(|p| p.name == "Health" && p.direction == PinDirection::Input)
            .expect("Health input pin missing");
        assert_eq!(data_in.pin_type.category, PinCategory::Float);

        let data_out = node
            .pins
            .iter()
            .find(|p| p.name == "Health" && p.direction == PinDirection::Output)
            .expect("Health output pin missing");
        assert_eq!(data_out.pin_type.category, PinCategory::Float);
    }

    #[test]
    fn cast_to_has_object_input_and_cast_success_failed_exec_outputs() {
        let node = cast_to("CastToCharacter", "Character", "/Script/Engine.Character");
        assert_eq!(node.class, "/Script/BlueprintGraph.K2Node_DynamicCast");
        let target_type = node
            .properties
            .get("TargetType")
            .expect("TargetType missing");
        assert_eq!(target_type, "Class'/Script/Engine.Character'");

        let obj_pin = node.find_pin("Object").expect("Object pin missing");
        assert_eq!(obj_pin.direction, PinDirection::Input);

        let success = node
            .find_pin("Cast Success")
            .expect("Cast Success pin missing");
        assert_eq!(success.direction, PinDirection::Output);
        assert_eq!(success.pin_type.category, PinCategory::Exec);

        let failed = node
            .find_pin("Cast Failed")
            .expect("Cast Failed pin missing");
        assert_eq!(failed.direction, PinDirection::Output);
        assert_eq!(failed.pin_type.category, PinCategory::Exec);

        let as_char = node
            .find_pin("As Character")
            .expect("As Character output pin missing");
        assert_eq!(as_char.direction, PinDirection::Output);
    }

    #[test]
    fn array_get_has_array_and_index_inputs_and_return_value_output() {
        let node = array_get("GetItem", PinType::int());
        assert_eq!(node.class, "/Script/BlueprintGraph.K2Node_CallFunction");
        let func_ref = node
            .properties
            .get("FunctionReference")
            .expect("FunctionReference missing");
        assert!(func_ref.contains("Array_Get"));

        let array_pin = node
            .find_pin("TargetArray")
            .expect("TargetArray pin missing");
        assert_eq!(array_pin.direction, PinDirection::Input);
        assert_eq!(array_pin.pin_type.container, ContainerType::Array);

        let index_pin = node.find_pin("Index").expect("Index pin missing");
        assert_eq!(index_pin.direction, PinDirection::Input);
        assert_eq!(index_pin.pin_type.category, PinCategory::Int);

        let ret_pin = node
            .find_pin("ReturnValue")
            .expect("ReturnValue pin missing");
        assert_eq!(ret_pin.direction, PinDirection::Output);
        assert_eq!(ret_pin.pin_type.category, PinCategory::Int);
    }
}
