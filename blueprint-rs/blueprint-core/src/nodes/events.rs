use crate::ast::{BpNode, Pin};
use crate::types::{ContainerType, PinCategory, PinDirection, PinType};

// Helper: creates the hidden OutputDelegate pin all events have
fn delegate_pin() -> Pin {
    let mut p = Pin::new(
        "OutputDelegate",
        PinDirection::Output,
        PinType {
            category: PinCategory::Delegate,
            sub_category: None,
            sub_category_object: None,
            container: ContainerType::None,
            is_reference: false,
            is_const: false,
        },
    );
    p.is_hidden = true;
    p.is_not_connectable = true;
    p
}

// Helper: create an actor event node
fn actor_event(name: &str, member_name: &str) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_Event", name)
        .with_property(
            "EventReference",
            format!(
                "(MemberParent=Class'/Script/Engine.Actor',MemberName=\"{}\")",
                member_name
            ),
        )
        .with_property("bOverrideFunction", "True")
        .with_pin(delegate_pin())
        .with_pin(Pin::exec_output("then"))
}

// Helper: create a pawn event node
fn pawn_event(name: &str, member_name: &str) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_Event", name)
        .with_property(
            "EventReference",
            format!(
                "(MemberParent=Class'/Script/Engine.Pawn',MemberName=\"{}\")",
                member_name
            ),
        )
        .with_property("bOverrideFunction", "True")
        .with_pin(delegate_pin())
        .with_pin(Pin::exec_output("then"))
}

// Helper: create a character event node
fn character_event(name: &str, member_name: &str) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_Event", name)
        .with_property(
            "EventReference",
            format!(
                "(MemberParent=Class'/Script/Engine.Character',MemberName=\"{}\")",
                member_name
            ),
        )
        .with_property("bOverrideFunction", "True")
        .with_pin(delegate_pin())
        .with_pin(Pin::exec_output("then"))
}

// ======= ACTOR EVENTS =======

/// Called when the game begins (BeginPlay)
pub fn begin_play(name: impl Into<String>) -> BpNode {
    let n = name.into();
    actor_event(&n, "ReceiveBeginPlay")
}

/// Called when the actor is being destroyed/removed (EndPlay)
/// Includes an EndPlayReason output (byte enum)
pub fn end_play(name: impl Into<String>) -> BpNode {
    let n = name.into();
    actor_event(&n, "ReceiveEndPlay").with_pin(Pin::data_output("EndPlayReason", PinType::byte()))
}

/// Called every frame (Tick)
/// Includes DeltaSeconds (float) output pin
pub fn tick(name: impl Into<String>) -> BpNode {
    let n = name.into();
    actor_event(&n, "ReceiveTick").with_pin(Pin::data_output("DeltaSeconds", PinType::float()))
}

/// Called when this actor overlaps another actor (BeginOverlap)
/// OtherActor output: object pin
pub fn begin_overlap(name: impl Into<String>) -> BpNode {
    let n = name.into();
    actor_event(&n, "ReceiveActorBeginOverlap").with_pin(Pin::data_output(
        "OtherActor",
        PinType::object("/Script/Engine.Actor"),
    ))
}

/// Called when overlap ends (EndOverlap)
pub fn end_overlap(name: impl Into<String>) -> BpNode {
    let n = name.into();
    actor_event(&n, "ReceiveActorEndOverlap").with_pin(Pin::data_output(
        "OtherActor",
        PinType::object("/Script/Engine.Actor"),
    ))
}

/// Called when this actor is hit by a line trace or sweep
/// Output pins: HitComponent, OtherActor, OtherComp, SelfMoved (bool),
/// HitLocation, HitNormal, NormalImpulse, Hit (struct)
pub fn hit(name: impl Into<String>) -> BpNode {
    let n = name.into();
    actor_event(&n, "ReceiveHit")
        .with_pin(Pin::data_output(
            "HitComponent",
            PinType::object("/Script/Engine.PrimitiveComponent"),
        ))
        .with_pin(Pin::data_output(
            "OtherActor",
            PinType::object("/Script/Engine.Actor"),
        ))
        .with_pin(Pin::data_output(
            "OtherComp",
            PinType::object("/Script/Engine.PrimitiveComponent"),
        ))
        .with_pin(Pin::data_output("SelfMoved", PinType::bool()))
        .with_pin(Pin::data_output(
            "HitLocation",
            PinType::struct_type("/Script/CoreUObject.Vector"),
        ))
        .with_pin(Pin::data_output(
            "HitNormal",
            PinType::struct_type("/Script/CoreUObject.Vector"),
        ))
        .with_pin(Pin::data_output(
            "NormalImpulse",
            PinType::struct_type("/Script/CoreUObject.Vector"),
        ))
        .with_pin(Pin::data_output(
            "Hit",
            PinType::struct_type("/Script/Engine.HitResult"),
        ))
}

/// Called when actor is clicked (OnClicked)
pub fn on_clicked(name: impl Into<String>) -> BpNode {
    let n = name.into();
    actor_event(&n, "ReceiveActorOnClicked").with_pin(Pin::data_output(
        "ButtonPressed",
        PinType::struct_type("/Script/InputCore.Key"),
    ))
}

/// Called when actor click is released (OnReleased)
pub fn on_released(name: impl Into<String>) -> BpNode {
    let n = name.into();
    actor_event(&n, "ReceiveActorOnReleased").with_pin(Pin::data_output(
        "ButtonReleased",
        PinType::struct_type("/Script/InputCore.Key"),
    ))
}

/// Called when actor is destroyed
pub fn destroyed(name: impl Into<String>) -> BpNode {
    let n = name.into();
    actor_event(&n, "ReceiveDestroyed")
}

// ======= PAWN EVENTS =======

/// Called when this pawn is possessed by a controller
pub fn possessed(name: impl Into<String>) -> BpNode {
    let n = name.into();
    pawn_event(&n, "ReceivePossessed").with_pin(Pin::data_output(
        "NewController",
        PinType::object("/Script/Engine.Controller"),
    ))
}

/// Called when this pawn is unpossessed
pub fn unpossessed(name: impl Into<String>) -> BpNode {
    let n = name.into();
    pawn_event(&n, "ReceiveUnPossessed").with_pin(Pin::data_output(
        "OldController",
        PinType::object("/Script/Engine.Controller"),
    ))
}

// ======= CHARACTER EVENTS =======

/// Called when character lands after falling
pub fn landed(name: impl Into<String>) -> BpNode {
    let n = name.into();
    character_event(&n, "ReceiveLanded").with_pin(Pin::data_output(
        "Hit",
        PinType::struct_type("/Script/Engine.HitResult"),
    ))
}

// ======= CUSTOM EVENTS =======

/// A user-defined custom event with a given name.
/// Custom events use K2Node_CustomEvent class (not K2Node_Event).
pub fn custom_event(node_name: impl Into<String>, event_name: impl Into<String>) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_CustomEvent", node_name)
        .with_property("CustomFunctionName", format!("\"{}\"", event_name.into()))
        .with_pin(delegate_pin())
        .with_pin(Pin::exec_output("then"))
}

/// Custom event with additional output pins (for passing data to connected nodes).
/// Params are inserted between the delegate pin and the "then" exec output.
pub fn custom_event_with_params(
    node_name: impl Into<String>,
    event_name: impl Into<String>,
    params: Vec<(&str, PinType)>,
) -> BpNode {
    let mut node = BpNode::new("/Script/BlueprintGraph.K2Node_CustomEvent", node_name)
        .with_property("CustomFunctionName", format!("\"{}\"", event_name.into()))
        .with_pin(delegate_pin());
    for (param_name, param_type) in params {
        node = node.with_pin(Pin::data_output(param_name, param_type));
    }
    node = node.with_pin(Pin::exec_output("then"));
    node
}

// ======= INPUT EVENTS =======

/// Input action event (fires when the named action mapping is triggered).
/// Pins: execute (exec in), Pressed (exec out), Released (exec out).
pub fn input_action(name: impl Into<String>, action_name: impl Into<String>) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_InputAction", name)
        .with_property("InputActionName", format!("\"{}\"", action_name.into()))
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("Pressed"))
        .with_pin(Pin::exec_output("Released"))
}

/// Input axis event (fires every tick for the named axis mapping).
/// Adds AxisValue (float out) pin.
pub fn input_axis(name: impl Into<String>, axis_name: impl Into<String>) -> BpNode {
    BpNode::new("/Script/BlueprintGraph.K2Node_InputAxisEvent", name)
        .with_property("InputAxisName", format!("\"{}\"", axis_name.into()))
        .with_pin(delegate_pin())
        .with_pin(Pin::data_output("AxisValue", PinType::float()))
        .with_pin(Pin::exec_output("then"))
}

/// Key press or release event node.
/// `pressed = true` maps to a Pressed event; `false` to a Released event.
pub fn key_event(name: impl Into<String>, key: impl Into<String>, pressed: bool) -> BpNode {
    let event_type = if pressed { "IE_Pressed" } else { "IE_Released" };
    BpNode::new("/Script/BlueprintGraph.K2Node_InputKeyEvent", name)
        .with_property("InputChord", format!("(Key=(KeyName=\"{}\"))", key.into()))
        .with_property("bConsumeInput", "True")
        .with_property("bExecuteWhenPaused", "False")
        .with_property("bOverrideParentBinding", "True")
        .with_property("InputKeyEvent", event_type)
        .with_pin(delegate_pin())
        .with_pin(Pin::exec_output("then"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_play_class_and_pins() {
        let node = begin_play("BP_BeginPlay");
        assert_eq!(node.class, "/Script/BlueprintGraph.K2Node_Event");
        assert_eq!(node.name, "BP_BeginPlay");
        assert!(node
            .properties
            .get("EventReference")
            .unwrap()
            .contains("ReceiveBeginPlay"));
        assert_eq!(node.properties.get("bOverrideFunction").unwrap(), "True");

        // Must have hidden delegate pin
        let delegate = node
            .find_pin("OutputDelegate")
            .expect("OutputDelegate pin missing");
        assert!(delegate.is_hidden);
        assert!(delegate.is_not_connectable);
        assert_eq!(delegate.direction, PinDirection::Output);
        assert_eq!(delegate.pin_type.category, PinCategory::Delegate);

        // Must have exec output
        assert!(node.find_pin("then").is_some(), "then pin missing");
    }

    #[test]
    fn test_tick_has_delta_seconds_float_output() {
        let node = tick("BP_Tick");
        let pin = node
            .find_pin("DeltaSeconds")
            .expect("DeltaSeconds pin missing");
        assert_eq!(pin.direction, PinDirection::Output);
        assert_eq!(pin.pin_type.category, PinCategory::Float);
    }

    #[test]
    fn test_custom_event_class_and_property() {
        let node = custom_event("MyEvent", "OnPlayerDied");
        assert_eq!(node.class, "/Script/BlueprintGraph.K2Node_CustomEvent");
        let func_name = node
            .properties
            .get("CustomFunctionName")
            .expect("CustomFunctionName property missing");
        assert!(
            func_name.contains("OnPlayerDied"),
            "CustomFunctionName should contain the event name, got: {}",
            func_name
        );
        assert!(
            node.find_pin("OutputDelegate").is_some(),
            "OutputDelegate pin missing"
        );
        assert!(node.find_pin("then").is_some(), "then pin missing");
        // Should NOT have EventReference or bOverrideFunction
        assert!(node.properties.get("EventReference").is_none());
        assert!(node.properties.get("bOverrideFunction").is_none());
    }

    #[test]
    fn test_begin_overlap_has_other_actor_output_pin() {
        let node = begin_overlap("BP_BeginOverlap");
        let pin = node.find_pin("OtherActor").expect("OtherActor pin missing");
        assert_eq!(pin.direction, PinDirection::Output);
        assert_eq!(pin.pin_type.category, PinCategory::Object);
        assert_eq!(
            pin.pin_type.sub_category_object.as_deref(),
            Some("/Script/Engine.Actor")
        );
    }

    #[test]
    fn test_tick_event_reference_and_parent() {
        let node = tick("Tick0");
        let event_ref = node.properties.get("EventReference").unwrap();
        assert!(event_ref.contains("ReceiveTick"));
        assert!(event_ref.contains("/Script/Engine.Actor"));
    }

    #[test]
    fn test_custom_event_with_params() {
        let node = custom_event_with_params(
            "MyParamEvent",
            "OnDamage",
            vec![
                ("DamageAmount", PinType::float()),
                ("DamageCauser", PinType::object("/Script/Engine.Actor")),
            ],
        );
        assert_eq!(node.class, "/Script/BlueprintGraph.K2Node_CustomEvent");
        assert!(node.find_pin("DamageAmount").is_some());
        assert!(node.find_pin("DamageCauser").is_some());
        assert!(node.find_pin("then").is_some());
        // Verify pin ordering: delegate, params, then
        let pin_names: Vec<&str> = node.pins.iter().map(|p| p.name.as_str()).collect();
        let delegate_pos = pin_names
            .iter()
            .position(|&n| n == "OutputDelegate")
            .unwrap();
        let then_pos = pin_names.iter().position(|&n| n == "then").unwrap();
        let damage_pos = pin_names.iter().position(|&n| n == "DamageAmount").unwrap();
        assert!(
            delegate_pos < damage_pos,
            "delegate should come before params"
        );
        assert!(damage_pos < then_pos, "params should come before then");
    }

    #[test]
    fn test_input_action_pins() {
        let node = input_action("JumpAction", "Jump");
        assert_eq!(node.class, "/Script/BlueprintGraph.K2Node_InputAction");
        assert!(node
            .properties
            .get("InputActionName")
            .unwrap()
            .contains("Jump"));
        assert!(node.find_pin("execute").is_some());
        assert!(node.find_pin("Pressed").is_some());
        assert!(node.find_pin("Released").is_some());
    }

    #[test]
    fn test_input_axis_pins() {
        let node = input_axis("MoveForwardAxis", "MoveForward");
        assert_eq!(node.class, "/Script/BlueprintGraph.K2Node_InputAxisEvent");
        assert!(node
            .properties
            .get("InputAxisName")
            .unwrap()
            .contains("MoveForward"));
        let axis_pin = node.find_pin("AxisValue").expect("AxisValue pin missing");
        assert_eq!(axis_pin.pin_type.category, PinCategory::Float);
        assert_eq!(axis_pin.direction, PinDirection::Output);
    }

    #[test]
    fn test_hit_output_pins() {
        let node = hit("HitEvent");
        assert!(node.find_pin("HitComponent").is_some());
        assert!(node.find_pin("OtherActor").is_some());
        assert!(node.find_pin("OtherComp").is_some());
        assert!(node.find_pin("SelfMoved").is_some());
        assert!(node.find_pin("HitLocation").is_some());
        assert!(node.find_pin("HitNormal").is_some());
        assert!(node.find_pin("NormalImpulse").is_some());
        assert!(node.find_pin("Hit").is_some());
        let self_moved = node.find_pin("SelfMoved").unwrap();
        assert_eq!(self_moved.pin_type.category, PinCategory::Boolean);
    }

    #[test]
    fn test_pawn_possessed_has_controller_pin() {
        let node = possessed("PossessedEvent");
        assert!(node
            .properties
            .get("EventReference")
            .unwrap()
            .contains("ReceivePossessed"));
        assert!(node
            .properties
            .get("EventReference")
            .unwrap()
            .contains("/Script/Engine.Pawn"));
        let pin = node
            .find_pin("NewController")
            .expect("NewController pin missing");
        assert_eq!(pin.pin_type.category, PinCategory::Object);
    }

    #[test]
    fn test_end_play_has_reason_pin() {
        let node = end_play("EndPlayEvent");
        let pin = node
            .find_pin("EndPlayReason")
            .expect("EndPlayReason pin missing");
        assert_eq!(pin.pin_type.category, PinCategory::Byte);
        assert_eq!(pin.direction, PinDirection::Output);
    }
}
