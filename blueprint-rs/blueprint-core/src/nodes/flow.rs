//! Factory functions that create Blueprint nodes for UE4 flow control.
//!
//! Each function returns a fully-configured [`BpNode`] whose class path, pin
//! names, pin directions, and default values match the UE4 T3D export format
//! for the corresponding `K2Node_*` class.

use crate::ast::*;
use crate::types::*;

// ── class-path constants ──────────────────────────────────────────────────────

const CLASS_BRANCH: &str = "/Script/BlueprintGraph.K2Node_IfThenElse";
const CLASS_FOR_LOOP: &str = "/Script/BlueprintGraph.K2Node_ForLoop";
const CLASS_FOR_LOOP_BREAK: &str = "/Script/BlueprintGraph.K2Node_ForLoopWithBreak";
const CLASS_SEQUENCE: &str = "/Script/BlueprintGraph.K2Node_ExecutionSequence";
const CLASS_WHILE_LOOP: &str = "/Script/BlueprintGraph.K2Node_WhileLoop";
const CLASS_DO_ONCE: &str = "/Script/BlueprintGraph.K2Node_DoOnce";
const CLASS_FLIP_FLOP: &str = "/Script/BlueprintGraph.K2Node_FlipFlop";
const CLASS_GATE: &str = "/Script/BlueprintGraph.K2Node_Gate";
const CLASS_MULTI_GATE: &str = "/Script/BlueprintGraph.K2Node_MultiGate";

// ── public factory functions ──────────────────────────────────────────────────

/// Branch node (if/then/else) — routes execution based on boolean condition.
///
/// UE4 class: `K2Node_IfThenElse`
///
/// Pins:
/// - `execute`   — exec input
/// - `Condition` — bool input
/// - `then`      — exec output (true branch)
/// - `else`      — exec output (false branch)
pub fn branch_node(name: impl Into<String>) -> BpNode {
    BpNode::new(CLASS_BRANCH, name)
        .with_pin(exec_in("execute"))
        .with_pin(bool_in("Condition"))
        .with_pin(exec_out("then"))
        .with_pin(exec_out("else"))
}

/// ForLoop node — iterates from `FirstIndex` to `LastIndex` (inclusive).
///
/// UE4 class: `K2Node_ForLoop`
///
/// Pins:
/// - `execute`    — exec input
/// - `FirstIndex` — int input (default: `first_default`)
/// - `LastIndex`  — int input (default: `last_default`)
/// - `LoopBody`   — exec output (runs each iteration)
/// - `Index`      — int output (current loop index)
/// - `Completed`  — exec output (runs after all iterations)
pub fn for_loop_node(
    name: impl Into<String>,
    first_default: i32,
    last_default: i32,
) -> BpNode {
    BpNode::new(CLASS_FOR_LOOP, name)
        .with_pin(exec_in("execute"))
        .with_pin(int_in("FirstIndex", first_default))
        .with_pin(int_in("LastIndex", last_default))
        .with_pin(exec_out("LoopBody"))
        .with_pin(int_out("Index"))
        .with_pin(exec_out("Completed"))
}

/// ForLoop with break — same as [`for_loop_node`] but adds a `Break` exec input
/// that terminates the loop early.
///
/// UE4 class: `K2Node_ForLoopWithBreak`
///
/// Pins: same as `K2Node_ForLoop` plus:
/// - `Break` — exec input
pub fn for_loop_with_break_node(
    name: impl Into<String>,
    first_default: i32,
    last_default: i32,
) -> BpNode {
    BpNode::new(CLASS_FOR_LOOP_BREAK, name)
        .with_pin(exec_in("execute"))
        .with_pin(int_in("FirstIndex", first_default))
        .with_pin(int_in("LastIndex", last_default))
        .with_pin(exec_in("Break"))
        .with_pin(exec_out("LoopBody"))
        .with_pin(int_out("Index"))
        .with_pin(exec_out("Completed"))
}

/// Sequence node — executes multiple output branches sequentially.
///
/// UE4 class: `K2Node_ExecutionSequence`
///
/// `output_count` determines how many exec outputs (`then_0`, `then_1`, …) are
/// created.  At least one output is always present; if `output_count` is 0 it
/// is clamped to 1.
///
/// Pins:
/// - `execute`  — exec input
/// - `then_0` … `then_{output_count-1}` — exec outputs
pub fn sequence_node(name: impl Into<String>, output_count: usize) -> BpNode {
    let count = output_count.max(1);
    let mut node = BpNode::new(CLASS_SEQUENCE, name).with_pin(exec_in("execute"));
    for i in 0..count {
        node = node.with_pin(exec_out(format!("then_{i}")));
    }
    node
}

/// WhileLoop node — loops while `Condition` is true.
///
/// UE4 class: `K2Node_WhileLoop`
///
/// Pins:
/// - `execute`   — exec input
/// - `Condition` — bool input
/// - `LoopBody`  — exec output (runs each iteration)
/// - `Completed` — exec output (runs when condition becomes false)
pub fn while_loop_node(name: impl Into<String>) -> BpNode {
    BpNode::new(CLASS_WHILE_LOOP, name)
        .with_pin(exec_in("execute"))
        .with_pin(bool_in("Condition"))
        .with_pin(exec_out("LoopBody"))
        .with_pin(exec_out("Completed"))
}

/// DoOnce node — executes only once until the `Reset` input is fired.
///
/// UE4 class: `K2Node_DoOnce`
///
/// Pins:
/// - `execute`      — exec input
/// - `Reset`        — exec input (re-arms the node)
/// - `then`         — exec output
/// - `bStartClosed` — bool input (if true the node starts in the closed/done state)
pub fn do_once_node(name: impl Into<String>) -> BpNode {
    BpNode::new(CLASS_DO_ONCE, name)
        .with_pin(exec_in("execute"))
        .with_pin(exec_in("Reset"))
        .with_pin(exec_out("then"))
        .with_pin(bool_in("bStartClosed"))
}

/// FlipFlop node — alternates between `A` and `B` exec outputs on each trigger.
///
/// UE4 class: `K2Node_FlipFlop`
///
/// Pins:
/// - `execute` — exec input
/// - `A`       — exec output (first and every odd trigger)
/// - `B`       — exec output (second and every even trigger)
/// - `IsA`     — bool output (true when `A` was just executed)
pub fn flip_flop_node(name: impl Into<String>) -> BpNode {
    BpNode::new(CLASS_FLIP_FLOP, name)
        .with_pin(exec_in("execute"))
        .with_pin(exec_out("A"))
        .with_pin(exec_out("B"))
        .with_pin(Pin::data_output("IsA", PinType::bool()))
}

/// Gate node — conditionally allows execution to pass through to `Exit`.
///
/// UE4 class: `K2Node_Gate`
///
/// Pins:
/// - `execute`      — exec input (passes through when gate is open)
/// - `Open`         — exec input (opens the gate)
/// - `Close`        — exec input (closes the gate)
/// - `Toggle`       — exec input (toggles open/closed)
/// - `Exit`         — exec output (fires when gate is open and `execute` fires)
/// - `bStartClosed` — bool input (initial state; default false = open)
pub fn gate_node(name: impl Into<String>) -> BpNode {
    BpNode::new(CLASS_GATE, name)
        .with_pin(exec_in("execute"))
        .with_pin(exec_in("Open"))
        .with_pin(exec_in("Close"))
        .with_pin(exec_in("Toggle"))
        .with_pin(exec_out("Exit"))
        .with_pin(bool_in("bStartClosed"))
}

/// MultiGate node — cycles through multiple exec outputs, one per trigger.
///
/// UE4 class: `K2Node_MultiGate`
///
/// `output_count` determines how many exec outputs (`Out_0`, `Out_1`, …) are
/// created.  At least one output is always present.
///
/// Pins:
/// - `execute`   — exec input
/// - `Reset`     — exec input (restarts the cycle)
/// - `bRandom`   — bool input (randomise output order)
/// - `bLoop`     — bool input (restart after all outputs fired)
/// - `StartIndex` — int input (which output to start on; default 0)
/// - `Out_0` … `Out_{output_count-1}` — exec outputs
pub fn multi_gate_node(name: impl Into<String>, output_count: usize) -> BpNode {
    let count = output_count.max(1);
    let mut node = BpNode::new(CLASS_MULTI_GATE, name)
        .with_pin(exec_in("execute"))
        .with_pin(exec_in("Reset"))
        .with_pin(bool_in("bRandom"))
        .with_pin(bool_in("bLoop"))
        .with_pin(int_in("StartIndex", 0));
    for i in 0..count {
        node = node.with_pin(exec_out(format!("Out_{i}")));
    }
    node
}

// ── private pin helpers ───────────────────────────────────────────────────────

/// Create an exec input pin (convenience shorthand).
fn exec_in(name: impl Into<String>) -> Pin {
    Pin::exec_input(name)
}

/// Create an exec output pin (convenience shorthand).
fn exec_out(name: impl Into<String>) -> Pin {
    Pin::exec_output(name)
}

/// Create a bool input pin.
fn bool_in(name: impl Into<String>) -> Pin {
    Pin::data_input(name, PinType::bool())
}

/// Create an int input pin with a default value.
fn int_in(name: impl Into<String>, default: i32) -> Pin {
    Pin::data_input(name, PinType::int()).with_default(default.to_string())
}

/// Create an int output pin.
fn int_out(name: impl Into<String>) -> Pin {
    Pin::data_output(name, PinType::int())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── branch_node ──────────────────────────────────────────────────────────

    #[test]
    fn branch_node_class_path() {
        let node = branch_node("Branch_0");
        assert_eq!(node.class, CLASS_BRANCH);
    }

    #[test]
    fn branch_node_name() {
        let node = branch_node("MyBranch");
        assert_eq!(node.name, "MyBranch");
    }

    #[test]
    fn branch_node_has_execute_exec_input() {
        let node = branch_node("B");
        let pin = node.find_pin("execute").expect("missing 'execute' pin");
        assert_eq!(pin.direction, PinDirection::Input);
    }

    #[test]
    fn branch_node_has_condition_bool_input() {
        let node = branch_node("B");
        let pin = node.find_pin("Condition").expect("missing 'Condition' pin");
        assert_eq!(pin.direction, PinDirection::Input);
    }

    #[test]
    fn branch_node_has_then_exec_output() {
        let node = branch_node("B");
        let pin = node.find_pin("then").expect("missing 'then' pin");
        assert_eq!(pin.direction, PinDirection::Output);
    }

    #[test]
    fn branch_node_has_else_exec_output() {
        let node = branch_node("B");
        let pin = node.find_pin("else").expect("missing 'else' pin");
        assert_eq!(pin.direction, PinDirection::Output);
    }

    #[test]
    fn branch_node_pin_count() {
        let node = branch_node("B");
        assert_eq!(node.pins.len(), 4);
    }

    // ── for_loop_node ────────────────────────────────────────────────────────

    #[test]
    fn for_loop_node_class_path() {
        let node = for_loop_node("Loop_0", 0, 9);
        assert_eq!(node.class, CLASS_FOR_LOOP);
    }

    #[test]
    fn for_loop_node_has_first_index_pin() {
        let node = for_loop_node("L", 0, 9);
        let pin = node.find_pin("FirstIndex").expect("missing 'FirstIndex' pin");
        assert_eq!(pin.direction, PinDirection::Input);
        assert_eq!(pin.default_value.as_deref(), Some("0"));
    }

    #[test]
    fn for_loop_node_has_last_index_pin() {
        let node = for_loop_node("L", 0, 9);
        let pin = node.find_pin("LastIndex").expect("missing 'LastIndex' pin");
        assert_eq!(pin.direction, PinDirection::Input);
        assert_eq!(pin.default_value.as_deref(), Some("9"));
    }

    #[test]
    fn for_loop_node_has_loop_body_exec_output() {
        let node = for_loop_node("L", 0, 9);
        let pin = node.find_pin("LoopBody").expect("missing 'LoopBody' pin");
        assert_eq!(pin.direction, PinDirection::Output);
    }

    #[test]
    fn for_loop_node_has_index_int_output() {
        let node = for_loop_node("L", 0, 9);
        let pin = node.find_pin("Index").expect("missing 'Index' pin");
        assert_eq!(pin.direction, PinDirection::Output);
    }

    #[test]
    fn for_loop_node_has_completed_exec_output() {
        let node = for_loop_node("L", 0, 9);
        let pin = node.find_pin("Completed").expect("missing 'Completed' pin");
        assert_eq!(pin.direction, PinDirection::Output);
    }

    #[test]
    fn for_loop_node_custom_defaults() {
        let node = for_loop_node("L", 5, 20);
        assert_eq!(
            node.find_pin("FirstIndex").unwrap().default_value.as_deref(),
            Some("5")
        );
        assert_eq!(
            node.find_pin("LastIndex").unwrap().default_value.as_deref(),
            Some("20")
        );
    }

    // ── for_loop_with_break_node ─────────────────────────────────────────────

    #[test]
    fn for_loop_with_break_node_class_path() {
        let node = for_loop_with_break_node("LB", 0, 9);
        assert_eq!(node.class, CLASS_FOR_LOOP_BREAK);
    }

    #[test]
    fn for_loop_with_break_node_has_break_exec_input() {
        let node = for_loop_with_break_node("LB", 0, 9);
        let pin = node.find_pin("Break").expect("missing 'Break' pin");
        assert_eq!(pin.direction, PinDirection::Input);
    }

    // ── sequence_node ────────────────────────────────────────────────────────

    #[test]
    fn sequence_node_class_path() {
        let node = sequence_node("Seq_0", 2);
        assert_eq!(node.class, CLASS_SEQUENCE);
    }

    #[test]
    fn sequence_node_three_outputs() {
        let node = sequence_node("Seq", 3);
        assert!(node.find_pin("then_0").is_some(), "missing 'then_0'");
        assert!(node.find_pin("then_1").is_some(), "missing 'then_1'");
        assert!(node.find_pin("then_2").is_some(), "missing 'then_2'");
    }

    #[test]
    fn sequence_node_outputs_are_exec_outputs() {
        let node = sequence_node("Seq", 3);
        for i in 0..3 {
            let name = format!("then_{i}");
            let pin = node.find_pin(&name).unwrap_or_else(|| panic!("missing '{name}'"));
            assert_eq!(pin.direction, PinDirection::Output);
        }
    }

    #[test]
    fn sequence_node_no_extra_then_pins() {
        let node = sequence_node("Seq", 3);
        assert!(node.find_pin("then_3").is_none());
    }

    #[test]
    fn sequence_node_has_execute_pin() {
        let node = sequence_node("Seq", 2);
        assert!(node.find_pin("execute").is_some());
    }

    #[test]
    fn sequence_node_zero_count_clamped_to_one() {
        let node = sequence_node("Seq", 0);
        assert!(node.find_pin("then_0").is_some());
    }

    // ── while_loop_node ──────────────────────────────────────────────────────

    #[test]
    fn while_loop_node_class_path() {
        let node = while_loop_node("While_0");
        assert_eq!(node.class, CLASS_WHILE_LOOP);
    }

    #[test]
    fn while_loop_node_has_condition_pin() {
        let node = while_loop_node("W");
        assert!(node.find_pin("Condition").is_some());
    }

    #[test]
    fn while_loop_node_has_loop_body_and_completed() {
        let node = while_loop_node("W");
        assert!(node.find_pin("LoopBody").is_some());
        assert!(node.find_pin("Completed").is_some());
    }

    // ── do_once_node ─────────────────────────────────────────────────────────

    #[test]
    fn do_once_node_class_path() {
        let node = do_once_node("Do_0");
        assert_eq!(node.class, CLASS_DO_ONCE);
    }

    #[test]
    fn do_once_node_has_reset_exec_input() {
        let node = do_once_node("D");
        let pin = node.find_pin("Reset").expect("missing 'Reset' pin");
        assert_eq!(pin.direction, PinDirection::Input);
    }

    #[test]
    fn do_once_node_has_b_start_closed_bool_input() {
        let node = do_once_node("D");
        let pin = node.find_pin("bStartClosed").expect("missing 'bStartClosed' pin");
        assert_eq!(pin.direction, PinDirection::Input);
    }

    // ── flip_flop_node ───────────────────────────────────────────────────────

    #[test]
    fn flip_flop_node_class_path() {
        let node = flip_flop_node("FF_0");
        assert_eq!(node.class, CLASS_FLIP_FLOP);
    }

    #[test]
    fn flip_flop_node_has_a_and_b_exec_outputs() {
        let node = flip_flop_node("FF");
        let a = node.find_pin("A").expect("missing 'A' pin");
        let b = node.find_pin("B").expect("missing 'B' pin");
        assert_eq!(a.direction, PinDirection::Output);
        assert_eq!(b.direction, PinDirection::Output);
    }

    #[test]
    fn flip_flop_node_has_is_a_bool_output() {
        let node = flip_flop_node("FF");
        let pin = node.find_pin("IsA").expect("missing 'IsA' pin");
        assert_eq!(pin.direction, PinDirection::Output);
    }

    // ── gate_node ────────────────────────────────────────────────────────────

    #[test]
    fn gate_node_class_path() {
        let node = gate_node("Gate_0");
        assert_eq!(node.class, CLASS_GATE);
    }

    #[test]
    fn gate_node_has_open_close_toggle_exec_inputs() {
        let node = gate_node("G");
        for name in &["Open", "Close", "Toggle"] {
            let pin = node.find_pin(name).unwrap_or_else(|| panic!("missing '{name}' pin"));
            assert_eq!(pin.direction, PinDirection::Input);
        }
    }

    #[test]
    fn gate_node_has_exit_exec_output() {
        let node = gate_node("G");
        let pin = node.find_pin("Exit").expect("missing 'Exit' pin");
        assert_eq!(pin.direction, PinDirection::Output);
    }

    #[test]
    fn gate_node_has_b_start_closed_bool_input() {
        let node = gate_node("G");
        let pin = node.find_pin("bStartClosed").expect("missing 'bStartClosed' pin");
        assert_eq!(pin.direction, PinDirection::Input);
    }

    // ── multi_gate_node ──────────────────────────────────────────────────────

    #[test]
    fn multi_gate_node_class_path() {
        let node = multi_gate_node("MG_0", 3);
        assert_eq!(node.class, CLASS_MULTI_GATE);
    }

    #[test]
    fn multi_gate_node_outputs_named_out_n() {
        let node = multi_gate_node("MG", 3);
        assert!(node.find_pin("Out_0").is_some());
        assert!(node.find_pin("Out_1").is_some());
        assert!(node.find_pin("Out_2").is_some());
        assert!(node.find_pin("Out_3").is_none());
    }

    #[test]
    fn multi_gate_node_zero_count_clamped_to_one() {
        let node = multi_gate_node("MG", 0);
        assert!(node.find_pin("Out_0").is_some());
    }
}
