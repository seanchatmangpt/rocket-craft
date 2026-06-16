use crate::ast::*;
use crate::types::*;

pub fn branch_node(name: impl Into<String>) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_IfThenElse", name)
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::data_input("Condition", PinType::bool()))
        .with_pin(Pin::exec_output("then"))
        .with_pin(Pin::exec_output("else"))
}
