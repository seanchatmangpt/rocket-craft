use crate::ast::{BpNode, Pin};
use crate::types::PinType;

// ─── Internal helpers ─────────────────────────────────────────────────────────

const CALL_FN:  &str = "/Script/BlueprintGraph.K2Node_CallFunction";
const COMM_BIN: &str = "/Script/BlueprintGraph.K2Node_CommutativeAssociativeBinaryOperator";
const MATH_LIB: &str = "/Script/Engine.KismetMathLibrary";
const SYS_LIB:  &str = "/Script/Engine.KismetSystemLibrary";

/// Build the FunctionReference property string for a `K2Node_CallFunction`.
fn fn_ref(library: &str, func_name: &str) -> String {
    format!(
        "(MemberParent=Class'{library}',MemberName=\"{func_name}\")",
        library = library,
        func_name = func_name,
    )
}

/// Create a `K2Node_CallFunction` wired to `KismetMathLibrary`.
fn math_call(name: impl Into<String>, func_name: &str) -> BpNode {
    BpNode::new(CALL_FN, name)
        .with_property("FunctionReference", fn_ref(MATH_LIB, func_name))
}

/// Create a `K2Node_CommutativeAssociativeBinaryOperator` wired to `KismetMathLibrary`.
fn comm_bin(name: impl Into<String>, func_name: &str) -> BpNode {
    BpNode::new(COMM_BIN, name)
        .with_property("FunctionReference", fn_ref(MATH_LIB, func_name))
}

// ─── Integer math ─────────────────────────────────────────────────────────────

/// Integer addition: `A + B → ReturnValue`
pub fn add_int(name: impl Into<String>) -> BpNode {
    comm_bin(name, "Add_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Integer subtraction: `A - B → ReturnValue`
pub fn subtract_int(name: impl Into<String>) -> BpNode {
    math_call(name, "Subtract_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Integer multiplication: `A * B → ReturnValue`
pub fn multiply_int(name: impl Into<String>) -> BpNode {
    comm_bin(name, "Multiply_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Integer division: `A / B → ReturnValue`
pub fn divide_int(name: impl Into<String>) -> BpNode {
    math_call(name, "Divide_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Integer modulo: `A % B → ReturnValue`
pub fn modulo_int(name: impl Into<String>) -> BpNode {
    math_call(name, "Percent_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Absolute value of an integer: `|A| → ReturnValue`
pub fn abs_int(name: impl Into<String>) -> BpNode {
    math_call(name, "Abs_Int")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Clamp an integer between `min` and `max`.
pub fn clamp_int(name: impl Into<String>, min: i32, max: i32) -> BpNode {
    math_call(name, "Clamp_Int")
        .with_pin(Pin::data_input("Value", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("Min", PinType::int()).with_default(&min.to_string()))
        .with_pin(Pin::data_input("Max", PinType::int()).with_default(&max.to_string()))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Minimum of two integers.
pub fn min_int(name: impl Into<String>) -> BpNode {
    math_call(name, "Min_Int")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Maximum of two integers.
pub fn max_int(name: impl Into<String>) -> BpNode {
    math_call(name, "Max_Int")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Random integer in [`min`, `max`] (inclusive).
pub fn random_int_in_range(name: impl Into<String>, min: i32, max: i32) -> BpNode {
    math_call(name, "RandomIntegerInRange")
        .with_pin(Pin::data_input("Min", PinType::int()).with_default(&min.to_string()))
        .with_pin(Pin::data_input("Max", PinType::int()).with_default(&max.to_string()))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Integer equality: `A == B → ReturnValue (bool)`
pub fn equal_int(name: impl Into<String>) -> BpNode {
    math_call(name, "EqualEqual_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

/// Integer inequality: `A != B → ReturnValue (bool)`
pub fn not_equal_int(name: impl Into<String>) -> BpNode {
    math_call(name, "NotEqual_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

/// Integer greater-than: `A > B → ReturnValue (bool)`
pub fn greater_int(name: impl Into<String>) -> BpNode {
    math_call(name, "Greater_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

/// Integer less-than: `A < B → ReturnValue (bool)`
pub fn less_int(name: impl Into<String>) -> BpNode {
    math_call(name, "Less_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

/// Integer greater-than-or-equal: `A >= B → ReturnValue (bool)`
pub fn greater_equal_int(name: impl Into<String>) -> BpNode {
    math_call(name, "GreaterEqual_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

/// Integer less-than-or-equal: `A <= B → ReturnValue (bool)`
pub fn less_equal_int(name: impl Into<String>) -> BpNode {
    math_call(name, "LessEqual_IntInt")
        .with_pin(Pin::data_input("A", PinType::int()).with_default("0"))
        .with_pin(Pin::data_input("B", PinType::int()).with_default("0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

// ─── Float math ───────────────────────────────────────────────────────────────

/// Float addition: `A + B → ReturnValue`
pub fn add_float(name: impl Into<String>) -> BpNode {
    comm_bin(name, "Add_FloatFloat")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("B", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Float subtraction: `A - B → ReturnValue`
pub fn subtract_float(name: impl Into<String>) -> BpNode {
    math_call(name, "Subtract_FloatFloat")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("B", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Float multiplication: `A * B → ReturnValue`
pub fn multiply_float(name: impl Into<String>) -> BpNode {
    comm_bin(name, "Multiply_FloatFloat")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("B", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Float division: `A / B → ReturnValue`
pub fn divide_float(name: impl Into<String>) -> BpNode {
    math_call(name, "Divide_FloatFloat")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("B", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Absolute value of a float: `|A| → ReturnValue`
pub fn abs_float(name: impl Into<String>) -> BpNode {
    math_call(name, "Abs")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Clamp a float between `min` and `max`.
pub fn clamp_float(name: impl Into<String>, min: f32, max: f32) -> BpNode {
    math_call(name, "FClamp")
        .with_pin(Pin::data_input("Value", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("Min", PinType::float()).with_default(&format!("{:.6}", min)))
        .with_pin(Pin::data_input("Max", PinType::float()).with_default(&format!("{:.6}", max)))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Minimum of two floats.
pub fn min_float(name: impl Into<String>) -> BpNode {
    math_call(name, "FMin")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("B", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Maximum of two floats.
pub fn max_float(name: impl Into<String>) -> BpNode {
    math_call(name, "FMax")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("B", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Linear interpolation: `Lerp(A, B, Alpha) → ReturnValue`
pub fn lerp_float(name: impl Into<String>) -> BpNode {
    math_call(name, "Lerp")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("B", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("Alpha", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Square root: `Sqrt(A) → ReturnValue`
pub fn sqrt(name: impl Into<String>) -> BpNode {
    math_call(name, "Sqrt")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Power: `Base ^ Exp → ReturnValue`
pub fn power(name: impl Into<String>) -> BpNode {
    math_call(name, "Power")
        .with_pin(Pin::data_input("Base", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("Exp", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Sine (radians): `Sin(A) → ReturnValue`
pub fn sin(name: impl Into<String>) -> BpNode {
    math_call(name, "Sin")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Cosine (radians): `Cos(A) → ReturnValue`
pub fn cos(name: impl Into<String>) -> BpNode {
    math_call(name, "Cos")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Floor (round down): `Floor(A) → ReturnValue`
pub fn floor(name: impl Into<String>) -> BpNode {
    math_call(name, "Floor")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Ceiling (round up): `Ceil(A) → ReturnValue`
pub fn ceil(name: impl Into<String>) -> BpNode {
    math_call(name, "Ceil")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Round to nearest integer: `Round(A) → ReturnValue`
pub fn round(name: impl Into<String>) -> BpNode {
    math_call(name, "Round")
        .with_pin(Pin::data_input("A", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", PinType::int()))
}

/// Random float in [0, 1).
pub fn random_float(name: impl Into<String>) -> BpNode {
    math_call(name, "RandomFloat")
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Random float in [`min`, `max`].
pub fn random_float_in_range(name: impl Into<String>, min: f32, max: f32) -> BpNode {
    math_call(name, "RandomFloatInRange")
        .with_pin(Pin::data_input("Min", PinType::float()).with_default(&format!("{:.6}", min)))
        .with_pin(Pin::data_input("Max", PinType::float()).with_default(&format!("{:.6}", max)))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

// ─── Boolean logic ────────────────────────────────────────────────────────────

/// Boolean AND: `A && B → ReturnValue`
pub fn bool_and(name: impl Into<String>) -> BpNode {
    math_call(name, "BooleanAND")
        .with_pin(Pin::data_input("A", PinType::bool()))
        .with_pin(Pin::data_input("B", PinType::bool()))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

/// Boolean OR: `A || B → ReturnValue`
pub fn bool_or(name: impl Into<String>) -> BpNode {
    math_call(name, "BooleanOR")
        .with_pin(Pin::data_input("A", PinType::bool()))
        .with_pin(Pin::data_input("B", PinType::bool()))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

/// Boolean NOT: `!A → ReturnValue`
pub fn bool_not(name: impl Into<String>) -> BpNode {
    math_call(name, "Not_PreBool")
        .with_pin(Pin::data_input("A", PinType::bool()))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

/// Boolean XOR: `A ^ B → ReturnValue`
pub fn bool_xor(name: impl Into<String>) -> BpNode {
    math_call(name, "BooleanXOR")
        .with_pin(Pin::data_input("A", PinType::bool()))
        .with_pin(Pin::data_input("B", PinType::bool()))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

// ─── Vector math ──────────────────────────────────────────────────────────────

fn vector_pin_type() -> PinType {
    PinType::struct_type("/Script/CoreUObject.Vector")
}

/// Construct a `FVector` from X, Y, Z floats.
pub fn make_vector(name: impl Into<String>) -> BpNode {
    math_call(name, "MakeVector")
        .with_pin(Pin::data_input("X", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("Y", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_input("Z", PinType::float()).with_default("0.0"))
        .with_pin(Pin::data_output("ReturnValue", vector_pin_type()))
}

/// Decompose a `FVector` into X, Y, Z floats.
pub fn break_vector(name: impl Into<String>) -> BpNode {
    math_call(name, "BreakVector")
        .with_pin(Pin::data_input("InVec", vector_pin_type()))
        .with_pin(Pin::data_output("X", PinType::float()))
        .with_pin(Pin::data_output("Y", PinType::float()))
        .with_pin(Pin::data_output("Z", PinType::float()))
}

/// Vector length (magnitude): `|V| → ReturnValue`
pub fn vector_length(name: impl Into<String>) -> BpNode {
    math_call(name, "VSize")
        .with_pin(Pin::data_input("A", vector_pin_type()))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Vector addition: `A + B → ReturnValue`
pub fn add_vector(name: impl Into<String>) -> BpNode {
    math_call(name, "Add_VectorVector")
        .with_pin(Pin::data_input("A", vector_pin_type()))
        .with_pin(Pin::data_input("B", vector_pin_type()))
        .with_pin(Pin::data_output("ReturnValue", vector_pin_type()))
}

/// Vector subtraction: `A - B → ReturnValue`
pub fn subtract_vector(name: impl Into<String>) -> BpNode {
    math_call(name, "Subtract_VectorVector")
        .with_pin(Pin::data_input("A", vector_pin_type()))
        .with_pin(Pin::data_input("B", vector_pin_type()))
        .with_pin(Pin::data_output("ReturnValue", vector_pin_type()))
}

/// Scale a vector by a float: `V * Scale → ReturnValue`
pub fn multiply_vector_float(name: impl Into<String>) -> BpNode {
    math_call(name, "Multiply_VectorFloat")
        .with_pin(Pin::data_input("A", vector_pin_type()))
        .with_pin(Pin::data_input("B", PinType::float()).with_default("1.0"))
        .with_pin(Pin::data_output("ReturnValue", vector_pin_type()))
}

/// Normalize a vector to unit length.
pub fn normalize_vector(name: impl Into<String>) -> BpNode {
    math_call(name, "Normal")
        .with_pin(Pin::data_input("A", vector_pin_type()))
        .with_pin(Pin::data_output("ReturnValue", vector_pin_type()))
}

/// Dot product: `A · B → ReturnValue`
pub fn dot_product(name: impl Into<String>) -> BpNode {
    math_call(name, "Dot_VectorVector")
        .with_pin(Pin::data_input("A", vector_pin_type()))
        .with_pin(Pin::data_input("B", vector_pin_type()))
        .with_pin(Pin::data_output("ReturnValue", PinType::float()))
}

/// Cross product: `A × B → ReturnValue`
pub fn cross_product(name: impl Into<String>) -> BpNode {
    math_call(name, "Cross_VectorVector")
        .with_pin(Pin::data_input("A", vector_pin_type()))
        .with_pin(Pin::data_input("B", vector_pin_type()))
        .with_pin(Pin::data_output("ReturnValue", vector_pin_type()))
}

// ─── Utility nodes ────────────────────────────────────────────────────────────

/// Print a string to the screen and log (KismetSystemLibrary::PrintString).
pub fn print_string(name: impl Into<String>, message: impl Into<String>) -> BpNode {
    BpNode::new(CALL_FN, name)
        .with_property("FunctionReference", fn_ref(SYS_LIB, "PrintString"))
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(Pin::data_input("InString", PinType::string()).with_default(&message.into()))
        .with_pin(Pin::data_input("bPrintToScreen", PinType::bool()).with_default("true"))
        .with_pin(Pin::data_input("bPrintToLog", PinType::bool()).with_default("true"))
        .with_pin(Pin::data_input("TextColor", PinType::struct_type("/Script/CoreUObject.LinearColor")))
        .with_pin(Pin::data_input("Duration", PinType::float()).with_default("2.000000"))
}

/// Add a time delay before continuing execution.
pub fn delay(name: impl Into<String>, duration: f32) -> BpNode {
    BpNode::new(CALL_FN, name)
        .with_property("FunctionReference", fn_ref(SYS_LIB, "Delay"))
        .with_pin(Pin::exec_input("execute"))
        .with_pin(Pin::exec_output("then"))
        .with_pin(
            Pin::data_input("Duration", PinType::float())
                .with_default(&format!("{:.6}", duration))
        )
}

/// Check whether an object reference is valid (non-null).
pub fn is_valid(name: impl Into<String>) -> BpNode {
    BpNode::new(CALL_FN, name)
        .with_property("FunctionReference", fn_ref(SYS_LIB, "IsValid"))
        .with_pin(Pin::data_input("Object", PinType::object("/Script/CoreUObject.Object")))
        .with_pin(Pin::data_output("ReturnValue", PinType::bool()))
}

// ─── Generic function call ────────────────────────────────────────────────────

/// Create a `K2Node_CallFunction` targeting any UE4 library function.
///
/// # Arguments
/// * `name`          — Blueprint node label.
/// * `member_parent` — Full UE4 class path, e.g. `"/Script/Engine.KismetMathLibrary"`.
/// * `member_name`   — Function name, e.g. `"Add_IntInt"`.
pub fn call_function(
    name: impl Into<String>,
    member_parent: impl Into<String>,
    member_name: impl Into<String>,
) -> BpNode {
    let func_ref = format!(
        "(MemberParent=Class'{mp}',MemberName=\"{mn}\")",
        mp = member_parent.into(),
        mn = member_name.into(),
    );
    BpNode::new(CALL_FN, name)
        .with_property("FunctionReference", func_ref)
}

// ─── Pin builder extension ────────────────────────────────────────────────────

/// Extension trait that adds a `with_default` builder method to [`Pin`].
#[allow(dead_code)]
trait PinExt {
    fn with_default(self, value: &str) -> Self;
}

impl PinExt for Pin {
    fn with_default(mut self, value: &str) -> Self {
        self.default_value = Some(value.to_string());
        self
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PinDirection;

    #[test]
    fn add_int_has_correct_class_and_pins() {
        let node = add_int("MyAdd");

        assert_eq!(node.class, COMM_BIN, "class should be CommutativeAssociativeBinaryOperator");

        let a_pin = node.find_pin("A").expect("should have pin A");
        assert!(matches!(a_pin.direction, PinDirection::Input));

        let b_pin = node.find_pin("B").expect("should have pin B");
        assert!(matches!(b_pin.direction, PinDirection::Input));

        let ret_pin = node.find_pin("ReturnValue").expect("should have pin ReturnValue");
        assert!(matches!(ret_pin.direction, PinDirection::Output));

        // FunctionReference must reference KismetMathLibrary / Add_IntInt
        let fn_ref_val = node.properties.get("FunctionReference")
            .expect("must have FunctionReference property");
        assert!(fn_ref_val.contains("KismetMathLibrary"), "must reference KismetMathLibrary");
        assert!(fn_ref_val.contains("Add_IntInt"), "must reference Add_IntInt");
    }

    #[test]
    fn print_string_has_exec_then_instring_pins() {
        let node = print_string("Log", "Hello World");

        assert_eq!(node.class, CALL_FN, "class should be K2Node_CallFunction");

        node.find_pin("execute").expect("should have execute pin");
        node.find_pin("then").expect("should have then pin");

        let in_str = node.find_pin("InString").expect("should have InString pin");
        assert!(matches!(in_str.direction, PinDirection::Input));
        assert_eq!(in_str.default_value.as_deref(), Some("Hello World"));

        let fn_ref_val = node.properties.get("FunctionReference")
            .expect("must have FunctionReference");
        assert!(fn_ref_val.contains("KismetSystemLibrary"), "must reference KismetSystemLibrary");
        assert!(fn_ref_val.contains("PrintString"), "must reference PrintString");
    }

    #[test]
    fn make_vector_has_xyz_inputs_and_return_value() {
        let node = make_vector("MakeVec");

        assert_eq!(node.class, CALL_FN);

        let x = node.find_pin("X").expect("should have pin X");
        assert!(matches!(x.direction, PinDirection::Input));

        let y = node.find_pin("Y").expect("should have pin Y");
        assert!(matches!(y.direction, PinDirection::Input));

        let z = node.find_pin("Z").expect("should have pin Z");
        assert!(matches!(z.direction, PinDirection::Input));

        let ret = node.find_pin("ReturnValue").expect("should have ReturnValue");
        assert!(matches!(ret.direction, PinDirection::Output));

        // ReturnValue should be a struct (FVector)
        let sub_obj = ret.pin_type.sub_category_object.as_deref()
            .expect("ReturnValue must have sub_category_object");
        assert!(sub_obj.contains("Vector"), "ReturnValue must be a Vector struct");
    }

    #[test]
    fn call_function_generic_creates_correct_property() {
        let node = call_function(
            "MyFunc",
            "/Script/Engine.KismetMathLibrary",
            "Sqrt",
        );

        assert_eq!(node.class, CALL_FN);
        let fn_ref_val = node.properties.get("FunctionReference").unwrap();
        assert!(fn_ref_val.contains("/Script/Engine.KismetMathLibrary"));
        assert!(fn_ref_val.contains("Sqrt"));
    }

    #[test]
    fn clamp_int_default_values_are_set() {
        let node = clamp_int("Clamp", 0, 100);
        let min_pin = node.find_pin("Min").expect("Min pin");
        let max_pin = node.find_pin("Max").expect("Max pin");
        assert_eq!(min_pin.default_value.as_deref(), Some("0"));
        assert_eq!(max_pin.default_value.as_deref(), Some("100"));
    }

    #[test]
    fn delay_default_duration_is_set() {
        let node = delay("Wait", 2.5);
        let dur_pin = node.find_pin("Duration").expect("Duration pin");
        let val = dur_pin.default_value.as_deref().expect("default set");
        // parse back to verify numeric accuracy
        let parsed: f32 = val.parse().expect("parseable float");
        assert!((parsed - 2.5).abs() < 1e-4, "duration mismatch: {}", val);
    }

    #[test]
    fn subtract_int_uses_call_function_not_comm_bin() {
        // Subtraction is not commutative so it uses K2Node_CallFunction
        let node = subtract_int("Sub");
        assert_eq!(node.class, CALL_FN);
    }

    #[test]
    fn bool_not_has_single_input_and_output() {
        let node = bool_not("Not");
        let a = node.find_pin("A").expect("pin A");
        assert!(matches!(a.direction, PinDirection::Input));
        let ret = node.find_pin("ReturnValue").expect("ReturnValue");
        assert!(matches!(ret.direction, PinDirection::Output));
    }
}
