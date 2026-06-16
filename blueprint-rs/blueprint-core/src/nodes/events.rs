use crate::ast::*;

pub fn begin_play_node(name: impl Into<String>) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_Event", name)
        .with_property("EventReference", "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"ReceiveBeginPlay\")")
        .with_property("bOverrideFunction", "True")
        .with_pin(Pin::exec_output("then"))
}
