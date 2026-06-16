use crate::ast::*;
use crate::types::*;

pub fn add_int_node(name: impl Into<String>) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_CommutativeAssociativeBinaryOperator", name)
        .with_property("FunctionReference", "(MemberParent=Class'/Script/Engine.KismetMathLibrary',MemberName=\"Add_IntInt\")")
        .with_pin(Pin::data_input("A", PinType::int()))
        .with_pin(Pin::data_input("B", PinType::int()))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}
