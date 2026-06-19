use crate::ast::*;
use crate::types::*;
use std::collections::HashMap;

/// Specification for a single pin in a node definition
#[derive(Debug, Clone)]
pub struct PinSpec {
    pub name: &'static str,
    pub direction: PinDirection,
    pub category: PinCategory,
    pub sub_category_object: Option<&'static str>,
    pub default_value: Option<&'static str>,
    pub is_optional: bool,
}

impl PinSpec {
    pub const fn exec_in(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Input,
            category: PinCategory::Exec,
            sub_category_object: None,
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn exec_out(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Output,
            category: PinCategory::Exec,
            sub_category_object: None,
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn int_in(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Input,
            category: PinCategory::Int,
            sub_category_object: None,
            default_value: Some("0"),
            is_optional: false,
        }
    }
    pub const fn int_out(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Output,
            category: PinCategory::Int,
            sub_category_object: None,
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn float_in(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Input,
            category: PinCategory::Float,
            sub_category_object: None,
            default_value: Some("0.0"),
            is_optional: false,
        }
    }
    pub const fn float_out(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Output,
            category: PinCategory::Float,
            sub_category_object: None,
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn bool_in(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Input,
            category: PinCategory::Boolean,
            sub_category_object: None,
            default_value: Some("False"),
            is_optional: false,
        }
    }
    pub const fn bool_out(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Output,
            category: PinCategory::Boolean,
            sub_category_object: None,
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn string_in(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Input,
            category: PinCategory::String,
            sub_category_object: None,
            default_value: Some(""),
            is_optional: false,
        }
    }
    pub const fn string_out(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Output,
            category: PinCategory::String,
            sub_category_object: None,
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn object_in(name: &'static str, class: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Input,
            category: PinCategory::Object,
            sub_category_object: Some(class),
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn object_out(name: &'static str, class: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Output,
            category: PinCategory::Object,
            sub_category_object: Some(class),
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn struct_in(name: &'static str, struct_path: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Input,
            category: PinCategory::Struct,
            sub_category_object: Some(struct_path),
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn struct_out(name: &'static str, struct_path: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Output,
            category: PinCategory::Struct,
            sub_category_object: Some(struct_path),
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn name_in(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Input,
            category: PinCategory::Name,
            sub_category_object: None,
            default_value: None,
            is_optional: false,
        }
    }
    pub const fn name_out(name: &'static str) -> Self {
        Self {
            name,
            direction: PinDirection::Output,
            category: PinCategory::Name,
            sub_category_object: None,
            default_value: None,
            is_optional: false,
        }
    }
}

const VECTOR: &str = "/Script/CoreUObject.Vector";
const ROTATOR: &str = "/Script/CoreUObject.Rotator";
const TRANSFORM: &str = "/Script/CoreUObject.Transform";
const LINEAR_COLOR: &str = "/Script/CoreUObject.LinearColor";

/// Category of a Blueprint node for organization
#[derive(Debug, Clone, PartialEq)]
pub enum NodeCategory {
    Math,
    String,
    FlowControl,
    Utilities,
    Actor,
    Component,
    Physics,
    Input,
    Rendering,
    Audio,
    AI,
    GameMode,
    SaveLoad,
    Network,
    Debug,
    Array,
    Map,
    Set,
    Struct,
    Custom,
}

/// Specification for a complete Blueprint node
#[derive(Debug, Clone)]
pub struct NodeSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: NodeCategory,
    pub node_class: &'static str,
    pub function_parent: Option<&'static str>,
    pub function_name: Option<&'static str>,
    pub pins: &'static [PinSpec],
    pub keywords: &'static [&'static str],
    pub tooltip: &'static str,
}

impl NodeSpec {
    pub fn create_node(&self, node_name: impl Into<String>) -> BpNode {
        let name = node_name.into();
        let mut node = BpNode::new(self.node_class, &name);
        if let (Some(parent), Some(func)) = (self.function_parent, self.function_name) {
            node.properties.insert(
                "FunctionReference".to_string(),
                format!("(MemberParent=Class'{}',MemberName=\"{}\")", parent, func),
            );
        }
        for spec in self.pins {
            let pin_type = PinType {
                category: spec.category.clone(),
                sub_category: None,
                sub_category_object: spec.sub_category_object.map(|s| s.to_string()),
                container: ContainerType::None,
                is_reference: false,
                is_const: false,
            };
            let mut pin = Pin::new(spec.name, spec.direction.clone(), pin_type);
            if let Some(dv) = spec.default_value {
                pin.default_value = Some(dv.to_string());
            }
            node.pins.push(pin);
        }
        node
    }
}

/// The global node registry
pub struct NodeRegistry {
    nodes: HashMap<&'static str, &'static NodeSpec>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            nodes: HashMap::new(),
        };
        for spec in ALL_NODES {
            reg.nodes.insert(spec.id, spec);
        }
        reg
    }
    pub fn get(&self, id: &str) -> Option<&&'static NodeSpec> {
        self.nodes.get(id)
    }
    pub fn search(&self, query: &str) -> Vec<&&'static NodeSpec> {
        let q = query.to_lowercase();
        let mut results: Vec<&&'static NodeSpec> = self
            .nodes
            .values()
            .filter(|spec| {
                spec.display_name.to_lowercase().contains(&q)
                    || spec.keywords.iter().any(|k| k.to_lowercase().contains(&q))
                    || spec.tooltip.to_lowercase().contains(&q)
                    || spec.id.to_lowercase().contains(&q)
            })
            .collect();
        results.sort_by_key(|s| s.display_name);
        results
    }
    pub fn by_category(&self, cat: &NodeCategory) -> Vec<&&'static NodeSpec> {
        let mut results: Vec<&&'static NodeSpec> =
            self.nodes.values().filter(|s| &s.category == cat).collect();
        results.sort_by_key(|s| s.display_name);
        results
    }
    pub fn create(&self, id: &str, node_name: impl Into<String>) -> Option<BpNode> {
        self.get(id).map(|spec| spec.create_node(node_name))
    }
    pub fn all_ids(&self) -> Vec<&'static str> {
        let mut ids: Vec<&'static str> = self.nodes.keys().copied().collect();
        ids.sort();
        ids
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

const CALL_FN: &str = "/Script/BlueprintGraph.K2Node_CallFunction";
const COMM_BIN: &str = "/Script/BlueprintGraph.K2Node_CommutativeAssociativeBinaryOperator";
const MATH_LIB: &str = "/Script/Engine.KismetMathLibrary";
const SYS_LIB: &str = "/Script/Engine.KismetSystemLibrary";
const ARRAY_LIB: &str = "/Script/Engine.KismetArrayLibrary";
const STRING_LIB: &str = "/Script/Engine.KismetStringLibrary";
const GAME_LIB: &str = "/Script/Engine.GameplayStatics";

pub static ALL_NODES: &[NodeSpec] = &[
    // === MATH - INTEGER ===
    NodeSpec {
        id: "add_int",
        display_name: "Add (Integer)",
        category: NodeCategory::Math,
        node_class: COMM_BIN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Add_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["add", "plus", "sum", "int", "+"],
        tooltip: "Add two integers",
    },
    NodeSpec {
        id: "subtract_int",
        display_name: "Subtract (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Subtract_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["subtract", "minus", "int", "-"],
        tooltip: "Subtract two integers",
    },
    NodeSpec {
        id: "multiply_int",
        display_name: "Multiply (Integer)",
        category: NodeCategory::Math,
        node_class: COMM_BIN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Multiply_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["multiply", "times", "int", "*"],
        tooltip: "Multiply two integers",
    },
    NodeSpec {
        id: "divide_int",
        display_name: "Divide (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Divide_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["divide", "int", "/"],
        tooltip: "Integer division (A / B)",
    },
    NodeSpec {
        id: "modulo_int",
        display_name: "Modulo (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Percent_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["modulo", "remainder", "mod", "%"],
        tooltip: "Integer modulo (A % B)",
    },
    NodeSpec {
        id: "abs_int",
        display_name: "Absolute (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Abs_Int"),
        pins: &[PinSpec::int_in("A"), PinSpec::int_out("ReturnValue")],
        keywords: &["abs", "absolute", "int"],
        tooltip: "Absolute value of an integer",
    },
    NodeSpec {
        id: "clamp_int",
        display_name: "Clamp (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Clamp"),
        pins: &[
            PinSpec::int_in("Value"),
            PinSpec::int_in("Min"),
            PinSpec::int_in("Max"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["clamp", "limit", "range", "int"],
        tooltip: "Clamp an integer to [Min, Max]",
    },
    NodeSpec {
        id: "min_int",
        display_name: "Min (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Min_Int"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["min", "minimum", "int"],
        tooltip: "Return the smaller of two integers",
    },
    NodeSpec {
        id: "max_int",
        display_name: "Max (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Max_Int"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["max", "maximum", "int"],
        tooltip: "Return the larger of two integers",
    },
    NodeSpec {
        id: "random_int_in_range",
        display_name: "Random Integer in Range",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("RandomIntegerInRange"),
        pins: &[
            PinSpec::int_in("Min"),
            PinSpec::int_in("Max"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["random", "rand", "int", "range"],
        tooltip: "Random integer in [Min, Max] inclusive",
    },
    NodeSpec {
        id: "equal_int",
        display_name: "Equal (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("EqualEqual_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["equal", "==", "int", "compare"],
        tooltip: "Return true if A == B",
    },
    NodeSpec {
        id: "not_equal_int",
        display_name: "Not Equal (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("NotEqual_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["not equal", "!=", "int"],
        tooltip: "Return true if A != B",
    },
    NodeSpec {
        id: "greater_int",
        display_name: "Greater Than (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Greater_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["greater", ">", "int", "compare"],
        tooltip: "Return true if A > B",
    },
    NodeSpec {
        id: "less_int",
        display_name: "Less Than (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Less_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["less", "<", "int", "compare"],
        tooltip: "Return true if A < B",
    },
    NodeSpec {
        id: "greater_equal_int",
        display_name: "Greater Than or Equal (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("GreaterEqual_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["greater equal", ">=", "int"],
        tooltip: "Return true if A >= B",
    },
    NodeSpec {
        id: "less_equal_int",
        display_name: "Less Than or Equal (Integer)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("LessEqual_IntInt"),
        pins: &[
            PinSpec::int_in("A"),
            PinSpec::int_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["less equal", "<=", "int"],
        tooltip: "Return true if A <= B",
    },
    // === MATH - FLOAT ===
    NodeSpec {
        id: "add_float",
        display_name: "Add (Float)",
        category: NodeCategory::Math,
        node_class: COMM_BIN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Add_FloatFloat"),
        pins: &[
            PinSpec::float_in("A"),
            PinSpec::float_in("B"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["add", "plus", "float", "+"],
        tooltip: "Add two floats",
    },
    NodeSpec {
        id: "subtract_float",
        display_name: "Subtract (Float)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Subtract_FloatFloat"),
        pins: &[
            PinSpec::float_in("A"),
            PinSpec::float_in("B"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["subtract", "minus", "float", "-"],
        tooltip: "Subtract two floats",
    },
    NodeSpec {
        id: "multiply_float",
        display_name: "Multiply (Float)",
        category: NodeCategory::Math,
        node_class: COMM_BIN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Multiply_FloatFloat"),
        pins: &[
            PinSpec::float_in("A"),
            PinSpec::float_in("B"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["multiply", "times", "float", "*"],
        tooltip: "Multiply two floats",
    },
    NodeSpec {
        id: "divide_float",
        display_name: "Divide (Float)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Divide_FloatFloat"),
        pins: &[
            PinSpec::float_in("A"),
            PinSpec::float_in("B"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["divide", "float", "/"],
        tooltip: "Float division (A / B)",
    },
    NodeSpec {
        id: "abs_float",
        display_name: "Absolute (Float)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Abs"),
        pins: &[PinSpec::float_in("A"), PinSpec::float_out("ReturnValue")],
        keywords: &["abs", "absolute", "float"],
        tooltip: "Absolute value of a float",
    },
    NodeSpec {
        id: "clamp_float",
        display_name: "Clamp (Float)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("FClamp"),
        pins: &[
            PinSpec::float_in("Value"),
            PinSpec::float_in("Min"),
            PinSpec::float_in("Max"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["clamp", "limit", "range", "float"],
        tooltip: "Clamp a float to [Min, Max]",
    },
    NodeSpec {
        id: "min_float",
        display_name: "Min (Float)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("FMin"),
        pins: &[
            PinSpec::float_in("A"),
            PinSpec::float_in("B"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["min", "minimum", "float"],
        tooltip: "Return the smaller of two floats",
    },
    NodeSpec {
        id: "max_float",
        display_name: "Max (Float)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("FMax"),
        pins: &[
            PinSpec::float_in("A"),
            PinSpec::float_in("B"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["max", "maximum", "float"],
        tooltip: "Return the larger of two floats",
    },
    NodeSpec {
        id: "lerp",
        display_name: "Lerp (Float)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Lerp"),
        pins: &[
            PinSpec::float_in("A"),
            PinSpec::float_in("B"),
            PinSpec::float_in("Alpha"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["lerp", "interpolate", "blend", "float"],
        tooltip: "Linear interpolation between A and B by Alpha",
    },
    NodeSpec {
        id: "sqrt",
        display_name: "Square Root",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Sqrt"),
        pins: &[PinSpec::float_in("A"), PinSpec::float_out("ReturnValue")],
        keywords: &["sqrt", "square root", "float"],
        tooltip: "Square root of A",
    },
    NodeSpec {
        id: "power",
        display_name: "Power",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Power"),
        pins: &[
            PinSpec::float_in("Base"),
            PinSpec::float_in("Exp"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["power", "exponent", "pow"],
        tooltip: "Raise Base to the power of Exp",
    },
    NodeSpec {
        id: "sin",
        display_name: "Sin",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Sin"),
        pins: &[PinSpec::float_in("A"), PinSpec::float_out("ReturnValue")],
        keywords: &["sin", "sine", "trig"],
        tooltip: "Sine of A (radians)",
    },
    NodeSpec {
        id: "cos",
        display_name: "Cos",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Cos"),
        pins: &[PinSpec::float_in("A"), PinSpec::float_out("ReturnValue")],
        keywords: &["cos", "cosine", "trig"],
        tooltip: "Cosine of A (radians)",
    },
    NodeSpec {
        id: "tan",
        display_name: "Tan",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Tan"),
        pins: &[PinSpec::float_in("A"), PinSpec::float_out("ReturnValue")],
        keywords: &["tan", "tangent", "trig"],
        tooltip: "Tangent of A (radians)",
    },
    NodeSpec {
        id: "floor",
        display_name: "Floor",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Floor"),
        pins: &[PinSpec::float_in("A"), PinSpec::int_out("ReturnValue")],
        keywords: &["floor", "round down"],
        tooltip: "Round float down to nearest integer",
    },
    NodeSpec {
        id: "ceil",
        display_name: "Ceil",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Ceil"),
        pins: &[PinSpec::float_in("A"), PinSpec::int_out("ReturnValue")],
        keywords: &["ceil", "ceiling", "round up"],
        tooltip: "Round float up to nearest integer",
    },
    NodeSpec {
        id: "round",
        display_name: "Round",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Round"),
        pins: &[PinSpec::float_in("A"), PinSpec::int_out("ReturnValue")],
        keywords: &["round", "float", "int"],
        tooltip: "Round float to nearest integer",
    },
    NodeSpec {
        id: "random_float",
        display_name: "Random Float",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("RandomFloat"),
        pins: &[PinSpec::float_out("ReturnValue")],
        keywords: &["random", "rand", "float"],
        tooltip: "Random float in [0, 1)",
    },
    NodeSpec {
        id: "random_float_in_range",
        display_name: "Random Float in Range",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("RandomFloatInRange"),
        pins: &[
            PinSpec::float_in("Min"),
            PinSpec::float_in("Max"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["random", "rand", "float", "range"],
        tooltip: "Random float in [Min, Max]",
    },
    NodeSpec {
        id: "equal_float",
        display_name: "Equal (Float)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("EqualEqual_FloatFloat"),
        pins: &[
            PinSpec::float_in("A"),
            PinSpec::float_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["equal", "==", "float"],
        tooltip: "Return true if A == B (float)",
    },
    // === MATH - VECTOR ===
    NodeSpec {
        id: "make_vector",
        display_name: "Make Vector",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("MakeVector"),
        pins: &[
            PinSpec::float_in("X"),
            PinSpec::float_in("Y"),
            PinSpec::float_in("Z"),
            PinSpec::struct_out("ReturnValue", VECTOR),
        ],
        keywords: &["make", "vector", "xyz", "construct"],
        tooltip: "Construct a FVector from X, Y, Z components",
    },
    NodeSpec {
        id: "break_vector",
        display_name: "Break Vector",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("BreakVector"),
        pins: &[
            PinSpec::struct_in("InVec", VECTOR),
            PinSpec::float_out("X"),
            PinSpec::float_out("Y"),
            PinSpec::float_out("Z"),
        ],
        keywords: &["break", "vector", "xyz", "decompose"],
        tooltip: "Decompose a FVector into X, Y, Z floats",
    },
    NodeSpec {
        id: "add_vector",
        display_name: "Add (Vector)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Add_VectorVector"),
        pins: &[
            PinSpec::struct_in("A", VECTOR),
            PinSpec::struct_in("B", VECTOR),
            PinSpec::struct_out("ReturnValue", VECTOR),
        ],
        keywords: &["add", "vector", "+"],
        tooltip: "Add two vectors component-wise",
    },
    NodeSpec {
        id: "subtract_vector",
        display_name: "Subtract (Vector)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Subtract_VectorVector"),
        pins: &[
            PinSpec::struct_in("A", VECTOR),
            PinSpec::struct_in("B", VECTOR),
            PinSpec::struct_out("ReturnValue", VECTOR),
        ],
        keywords: &["subtract", "vector", "-"],
        tooltip: "Subtract two vectors component-wise",
    },
    NodeSpec {
        id: "multiply_vector_float",
        display_name: "Multiply (Vector * Float)",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Multiply_VectorFloat"),
        pins: &[
            PinSpec::struct_in("A", VECTOR),
            PinSpec::float_in("B"),
            PinSpec::struct_out("ReturnValue", VECTOR),
        ],
        keywords: &["scale", "multiply", "vector", "float", "*"],
        tooltip: "Scale a vector by a float",
    },
    NodeSpec {
        id: "vector_length",
        display_name: "Vector Length",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("VSize"),
        pins: &[
            PinSpec::struct_in("A", VECTOR),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["length", "magnitude", "size", "vector", "vsize"],
        tooltip: "Get the magnitude (length) of a vector",
    },
    NodeSpec {
        id: "normalize_vector",
        display_name: "Normalize Vector",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Normal"),
        pins: &[
            PinSpec::struct_in("A", VECTOR),
            PinSpec::struct_out("ReturnValue", VECTOR),
        ],
        keywords: &["normalize", "normal", "unit", "vector"],
        tooltip: "Normalize a vector to unit length",
    },
    NodeSpec {
        id: "dot_product",
        display_name: "Dot Product",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Dot_VectorVector"),
        pins: &[
            PinSpec::struct_in("A", VECTOR),
            PinSpec::struct_in("B", VECTOR),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["dot", "dot product", "vector"],
        tooltip: "Dot product of two vectors",
    },
    NodeSpec {
        id: "cross_product",
        display_name: "Cross Product",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Cross_VectorVector"),
        pins: &[
            PinSpec::struct_in("A", VECTOR),
            PinSpec::struct_in("B", VECTOR),
            PinSpec::struct_out("ReturnValue", VECTOR),
        ],
        keywords: &["cross", "cross product", "vector", "perpendicular"],
        tooltip: "Cross product of two vectors",
    },
    NodeSpec {
        id: "vector_lerp",
        display_name: "Vector Lerp",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("VLerp"),
        pins: &[
            PinSpec::struct_in("A", VECTOR),
            PinSpec::struct_in("B", VECTOR),
            PinSpec::float_in("Alpha"),
            PinSpec::struct_out("ReturnValue", VECTOR),
        ],
        keywords: &["lerp", "interpolate", "blend", "vector"],
        tooltip: "Linearly interpolate between two vectors",
    },
    NodeSpec {
        id: "find_look_at_rotation",
        display_name: "Find Look At Rotation",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("FindLookAtRotation"),
        pins: &[
            PinSpec::struct_in("Start", VECTOR),
            PinSpec::struct_in("Target", VECTOR),
            PinSpec::struct_out("ReturnValue", ROTATOR),
        ],
        keywords: &["look at", "rotation", "face", "aim", "vector"],
        tooltip: "Find the rotation needed to look from Start to Target",
    },
    // === MATH - BOOL ===
    NodeSpec {
        id: "bool_and",
        display_name: "Boolean AND",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("BooleanAND"),
        pins: &[
            PinSpec::bool_in("A"),
            PinSpec::bool_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["and", "bool", "&&", "logic"],
        tooltip: "Boolean AND (A && B)",
    },
    NodeSpec {
        id: "bool_or",
        display_name: "Boolean OR",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("BooleanOR"),
        pins: &[
            PinSpec::bool_in("A"),
            PinSpec::bool_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["or", "bool", "||", "logic"],
        tooltip: "Boolean OR (A || B)",
    },
    NodeSpec {
        id: "bool_not",
        display_name: "Boolean NOT",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Not_PreBool"),
        pins: &[PinSpec::bool_in("A"), PinSpec::bool_out("ReturnValue")],
        keywords: &["not", "bool", "!", "negate"],
        tooltip: "Boolean NOT (!A)",
    },
    NodeSpec {
        id: "bool_xor",
        display_name: "Boolean XOR",
        category: NodeCategory::Math,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("BooleanXOR"),
        pins: &[
            PinSpec::bool_in("A"),
            PinSpec::bool_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["xor", "bool", "exclusive or"],
        tooltip: "Boolean XOR (A ^ B)",
    },
    // === STRING ===
    NodeSpec {
        id: "append_string",
        display_name: "Append (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Concat_StrStr"),
        pins: &[
            PinSpec::string_in("A"),
            PinSpec::string_in("B"),
            PinSpec::string_out("ReturnValue"),
        ],
        keywords: &["append", "concat", "join", "string", "+"],
        tooltip: "Concatenate two strings",
    },
    NodeSpec {
        id: "int_to_string",
        display_name: "Int to String",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Conv_IntToString"),
        pins: &[PinSpec::int_in("InInt"), PinSpec::string_out("ReturnValue")],
        keywords: &["int", "string", "convert", "to string"],
        tooltip: "Convert an integer to a string",
    },
    NodeSpec {
        id: "float_to_string",
        display_name: "Float to String",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Conv_FloatToString"),
        pins: &[
            PinSpec::float_in("InFloat"),
            PinSpec::string_out("ReturnValue"),
        ],
        keywords: &["float", "string", "convert", "to string"],
        tooltip: "Convert a float to a string",
    },
    NodeSpec {
        id: "bool_to_string",
        display_name: "Bool to String",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("Conv_BoolToString"),
        pins: &[
            PinSpec::bool_in("InBool"),
            PinSpec::string_out("ReturnValue"),
        ],
        keywords: &["bool", "string", "convert", "to string"],
        tooltip: "Convert a boolean to a string",
    },
    NodeSpec {
        id: "contains_string",
        display_name: "Contains (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("Contains"),
        pins: &[
            PinSpec::string_in("SearchIn"),
            PinSpec::string_in("Substring"),
            PinSpec::bool_in("bUseCase"),
            PinSpec::bool_in("bSearchFromEnd"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["contains", "find", "search", "string", "substring"],
        tooltip: "Check if a string contains a substring",
    },
    NodeSpec {
        id: "string_length",
        display_name: "String Length",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("Len"),
        pins: &[PinSpec::string_in("S"), PinSpec::int_out("ReturnValue")],
        keywords: &["length", "len", "count", "string"],
        tooltip: "Return the number of characters in a string",
    },
    NodeSpec {
        id: "to_upper",
        display_name: "To Upper (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("ToUpper"),
        pins: &[
            PinSpec::string_in("SourceString"),
            PinSpec::string_out("ReturnValue"),
        ],
        keywords: &["upper", "uppercase", "string"],
        tooltip: "Convert string to uppercase",
    },
    NodeSpec {
        id: "to_lower",
        display_name: "To Lower (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("ToLower"),
        pins: &[
            PinSpec::string_in("SourceString"),
            PinSpec::string_out("ReturnValue"),
        ],
        keywords: &["lower", "lowercase", "string"],
        tooltip: "Convert string to lowercase",
    },
    NodeSpec {
        id: "replace_string",
        display_name: "Replace (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("Replace"),
        pins: &[
            PinSpec::string_in("SourceString"),
            PinSpec::string_in("From"),
            PinSpec::string_in("To"),
            PinSpec::string_out("ReturnValue"),
        ],
        keywords: &["replace", "substitute", "string"],
        tooltip: "Replace all occurrences of From with To",
    },
    NodeSpec {
        id: "split_string",
        display_name: "Split (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("Split"),
        pins: &[
            PinSpec::string_in("SourceString"),
            PinSpec::string_in("Delimiter"),
            PinSpec::string_out("LeftS"),
            PinSpec::string_out("RightS"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["split", "divide", "string", "delimiter"],
        tooltip: "Split a string at the first occurrence of Delimiter",
    },
    NodeSpec {
        id: "string_left",
        display_name: "Left (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("Left"),
        pins: &[
            PinSpec::string_in("SourceString"),
            PinSpec::int_in("Count"),
            PinSpec::string_out("ReturnValue"),
        ],
        keywords: &["left", "substring", "string"],
        tooltip: "Return the leftmost Count characters",
    },
    NodeSpec {
        id: "string_right",
        display_name: "Right (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("Right"),
        pins: &[
            PinSpec::string_in("SourceString"),
            PinSpec::int_in("Count"),
            PinSpec::string_out("ReturnValue"),
        ],
        keywords: &["right", "substring", "string"],
        tooltip: "Return the rightmost Count characters",
    },
    NodeSpec {
        id: "string_mid",
        display_name: "Mid (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("Mid"),
        pins: &[
            PinSpec::string_in("SourceString"),
            PinSpec::int_in("Start"),
            PinSpec::int_in("Count"),
            PinSpec::string_out("ReturnValue"),
        ],
        keywords: &["mid", "substring", "string", "slice"],
        tooltip: "Return a substring starting at Start with length Count",
    },
    NodeSpec {
        id: "equal_string",
        display_name: "Equal (String)",
        category: NodeCategory::String,
        node_class: CALL_FN,
        function_parent: Some(STRING_LIB),
        function_name: Some("EqualEqual_StrStr"),
        pins: &[
            PinSpec::string_in("A"),
            PinSpec::string_in("B"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["equal", "string", "==", "compare"],
        tooltip: "Return true if two strings are equal",
    },
    // === FLOW CONTROL ===
    NodeSpec {
        id: "branch",
        display_name: "Branch",
        category: NodeCategory::FlowControl,
        node_class: "/Script/BlueprintGraph.K2Node_IfThenElse",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::bool_in("Condition"),
            PinSpec::exec_out("True"),
            PinSpec::exec_out("False"),
        ],
        keywords: &["if", "else", "branch", "condition", "bool"],
        tooltip: "Branch execution based on a boolean condition",
    },
    NodeSpec {
        id: "for_loop",
        display_name: "For Loop",
        category: NodeCategory::FlowControl,
        node_class: "/Script/BlueprintGraph.K2Node_MacroInstance",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::int_in("FirstIndex"),
            PinSpec::int_in("LastIndex"),
            PinSpec::exec_out("LoopBody"),
            PinSpec::int_out("Index"),
            PinSpec::exec_out("Completed"),
        ],
        keywords: &["for", "loop", "iterate", "count"],
        tooltip: "Execute LoopBody for each integer from FirstIndex to LastIndex",
    },
    NodeSpec {
        id: "for_loop_with_break",
        display_name: "For Loop with Break",
        category: NodeCategory::FlowControl,
        node_class: "/Script/BlueprintGraph.K2Node_MacroInstance",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::exec_in("Break"),
            PinSpec::int_in("FirstIndex"),
            PinSpec::int_in("LastIndex"),
            PinSpec::exec_out("LoopBody"),
            PinSpec::int_out("Index"),
            PinSpec::exec_out("Completed"),
        ],
        keywords: &["for", "loop", "break", "iterate"],
        tooltip: "For loop that can be broken out of early",
    },
    NodeSpec {
        id: "sequence",
        display_name: "Sequence",
        category: NodeCategory::FlowControl,
        node_class: "/Script/BlueprintGraph.K2Node_ExecutionSequence",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::exec_out("Then 0"),
            PinSpec::exec_out("Then 1"),
        ],
        keywords: &["sequence", "then", "order", "flow"],
        tooltip: "Execute multiple outputs in sequence",
    },
    NodeSpec {
        id: "do_once",
        display_name: "Do Once",
        category: NodeCategory::FlowControl,
        node_class: "/Script/BlueprintGraph.K2Node_MacroInstance",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::exec_in("Reset"),
            PinSpec::bool_in("bStartClosed"),
            PinSpec::exec_out("Completed"),
        ],
        keywords: &["do once", "once", "single", "first"],
        tooltip: "Execute output only the first time (until Reset)",
    },
    NodeSpec {
        id: "flip_flop",
        display_name: "Flip Flop",
        category: NodeCategory::FlowControl,
        node_class: "/Script/BlueprintGraph.K2Node_MacroInstance",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::exec_out("A"),
            PinSpec::exec_out("B"),
            PinSpec::bool_out("IsA"),
        ],
        keywords: &["flip flop", "toggle", "alternate", "switch"],
        tooltip: "Alternates between two execution outputs",
    },
    NodeSpec {
        id: "gate",
        display_name: "Gate",
        category: NodeCategory::FlowControl,
        node_class: "/Script/BlueprintGraph.K2Node_MacroInstance",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("Enter"),
            PinSpec::exec_in("Open"),
            PinSpec::exec_in("Close"),
            PinSpec::exec_in("Toggle"),
            PinSpec::bool_in("bStartClosed"),
            PinSpec::exec_out("Exit"),
        ],
        keywords: &["gate", "enable", "disable", "allow"],
        tooltip: "Gate that can be opened and closed to control flow",
    },
    NodeSpec {
        id: "delay",
        display_name: "Delay",
        category: NodeCategory::FlowControl,
        node_class: CALL_FN,
        function_parent: Some(SYS_LIB),
        function_name: Some("Delay"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::float_in("Duration"),
            PinSpec::exec_out("Completed"),
        ],
        keywords: &["delay", "wait", "timer", "pause", "sleep"],
        tooltip: "Add a time delay before continuing execution",
    },
    NodeSpec {
        id: "while_loop",
        display_name: "While Loop",
        category: NodeCategory::FlowControl,
        node_class: "/Script/BlueprintGraph.K2Node_MacroInstance",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::bool_in("Condition"),
            PinSpec::exec_out("LoopBody"),
            PinSpec::exec_out("Completed"),
        ],
        keywords: &["while", "loop", "condition"],
        tooltip: "Execute LoopBody while Condition is true",
    },
    NodeSpec {
        id: "set_timer_by_event",
        display_name: "Set Timer by Event",
        category: NodeCategory::FlowControl,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("SetTimerDelegate"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::float_in("Time"),
            PinSpec::bool_in("bLooping"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["timer", "delay", "repeat", "loop", "event"],
        tooltip: "Set a timer that fires a delegate after the given time",
    },
    NodeSpec {
        id: "clear_timer",
        display_name: "Clear Timer by Handle",
        category: NodeCategory::FlowControl,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("ClearAndInvalidateTimerHandle"),
        pins: &[PinSpec::exec_in("execute"), PinSpec::exec_out("then")],
        keywords: &["timer", "clear", "cancel", "stop"],
        tooltip: "Clear and invalidate a timer handle",
    },
    // === DEBUG ===
    NodeSpec {
        id: "print_string",
        display_name: "Print String",
        category: NodeCategory::Debug,
        node_class: CALL_FN,
        function_parent: Some(SYS_LIB),
        function_name: Some("PrintString"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::string_in("InString"),
            PinSpec::bool_in("bPrintToScreen"),
            PinSpec::bool_in("bPrintToLog"),
            PinSpec::struct_in("TextColor", LINEAR_COLOR),
            PinSpec::float_in("Duration"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["print", "log", "debug", "display", "screen", "string"],
        tooltip: "Print a string to the screen and log",
    },
    NodeSpec {
        id: "draw_debug_sphere",
        display_name: "Draw Debug Sphere",
        category: NodeCategory::Debug,
        node_class: CALL_FN,
        function_parent: Some(SYS_LIB),
        function_name: Some("DrawDebugSphere"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::struct_in("Center", VECTOR),
            PinSpec::float_in("Radius"),
            PinSpec::int_in("Segments"),
            PinSpec::struct_in("LineColor", LINEAR_COLOR),
            PinSpec::float_in("Duration"),
            PinSpec::float_in("Thickness"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["debug", "draw", "sphere", "visualize"],
        tooltip: "Draw a debug sphere in the world",
    },
    NodeSpec {
        id: "draw_debug_line",
        display_name: "Draw Debug Line",
        category: NodeCategory::Debug,
        node_class: CALL_FN,
        function_parent: Some(SYS_LIB),
        function_name: Some("DrawDebugLine"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::struct_in("LineStart", VECTOR),
            PinSpec::struct_in("LineEnd", VECTOR),
            PinSpec::struct_in("LineColor", LINEAR_COLOR),
            PinSpec::float_in("Duration"),
            PinSpec::float_in("Thickness"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["debug", "draw", "line", "visualize"],
        tooltip: "Draw a debug line in the world",
    },
    NodeSpec {
        id: "draw_debug_box",
        display_name: "Draw Debug Box",
        category: NodeCategory::Debug,
        node_class: CALL_FN,
        function_parent: Some(SYS_LIB),
        function_name: Some("DrawDebugBox"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::struct_in("Center", VECTOR),
            PinSpec::struct_in("Extent", VECTOR),
            PinSpec::struct_in("LineColor", LINEAR_COLOR),
            PinSpec::float_in("Duration"),
            PinSpec::float_in("Thickness"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["debug", "draw", "box", "visualize"],
        tooltip: "Draw a debug box in the world",
    },
    // === UTILITIES ===
    NodeSpec {
        id: "is_valid",
        display_name: "Is Valid",
        category: NodeCategory::Utilities,
        node_class: CALL_FN,
        function_parent: Some(SYS_LIB),
        function_name: Some("IsValid"),
        pins: &[
            PinSpec::object_in("Object", "/Script/CoreUObject.Object"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["valid", "null", "check", "exists"],
        tooltip: "Check whether an object reference is valid (non-null)",
    },
    NodeSpec {
        id: "get_world_delta_seconds",
        display_name: "Get World Delta Seconds",
        category: NodeCategory::Utilities,
        node_class: CALL_FN,
        function_parent: Some(SYS_LIB),
        function_name: Some("GetWorldDeltaSeconds"),
        pins: &[
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["delta", "time", "seconds", "frame"],
        tooltip: "Get the time elapsed since the last frame",
    },
    NodeSpec {
        id: "get_game_time_in_seconds",
        display_name: "Get Game Time in Seconds",
        category: NodeCategory::Utilities,
        node_class: CALL_FN,
        function_parent: Some(SYS_LIB),
        function_name: Some("GetGameTimeInSeconds"),
        pins: &[
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["time", "seconds", "game time", "elapsed"],
        tooltip: "Get the total game time elapsed in seconds",
    },
    NodeSpec {
        id: "get_real_time_seconds",
        display_name: "Get Real Time Seconds",
        category: NodeCategory::Utilities,
        node_class: CALL_FN,
        function_parent: Some(SYS_LIB),
        function_name: Some("GetRealTimeSeconds"),
        pins: &[
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["real time", "time", "seconds", "wall clock"],
        tooltip: "Get the real wall-clock time in seconds",
    },
    // === ACTOR ===
    NodeSpec {
        id: "destroy_actor",
        display_name: "Destroy Actor",
        category: NodeCategory::Actor,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("K2_DestroyActor"),
        pins: &[PinSpec::exec_in("execute"), PinSpec::exec_out("then")],
        keywords: &["destroy", "delete", "remove", "actor", "kill"],
        tooltip: "Destroy this actor",
    },
    NodeSpec {
        id: "set_actor_location",
        display_name: "Set Actor Location",
        category: NodeCategory::Actor,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("K2_SetActorLocation"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::struct_in("NewLocation", VECTOR),
            PinSpec::bool_in("bSweep"),
            PinSpec::bool_in("bTeleport"),
            PinSpec::bool_out("ReturnValue"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["set", "location", "position", "move", "actor"],
        tooltip: "Set the actor's world position",
    },
    NodeSpec {
        id: "get_actor_location",
        display_name: "Get Actor Location",
        category: NodeCategory::Actor,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("K2_GetActorLocation"),
        pins: &[PinSpec::struct_out("ReturnValue", VECTOR)],
        keywords: &["get", "location", "position", "actor"],
        tooltip: "Get the actor's world position",
    },
    NodeSpec {
        id: "set_actor_rotation",
        display_name: "Set Actor Rotation",
        category: NodeCategory::Actor,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("K2_SetActorRotation"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::struct_in("NewRotation", ROTATOR),
            PinSpec::bool_in("bTeleportPhysics"),
            PinSpec::bool_out("ReturnValue"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["set", "rotation", "rotate", "actor"],
        tooltip: "Set the actor's world rotation",
    },
    NodeSpec {
        id: "get_actor_rotation",
        display_name: "Get Actor Rotation",
        category: NodeCategory::Actor,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("K2_GetActorRotation"),
        pins: &[PinSpec::struct_out("ReturnValue", ROTATOR)],
        keywords: &["get", "rotation", "actor"],
        tooltip: "Get the actor's world rotation",
    },
    NodeSpec {
        id: "set_actor_scale",
        display_name: "Set Actor Scale 3D",
        category: NodeCategory::Actor,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("SetActorScale3D"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::struct_in("NewScale3D", VECTOR),
            PinSpec::exec_out("then"),
        ],
        keywords: &["set", "scale", "size", "actor"],
        tooltip: "Set the actor's scale in 3D",
    },
    NodeSpec {
        id: "get_actor_scale",
        display_name: "Get Actor Scale 3D",
        category: NodeCategory::Actor,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("GetActorScale3D"),
        pins: &[PinSpec::struct_out("ReturnValue", VECTOR)],
        keywords: &["get", "scale", "size", "actor"],
        tooltip: "Get the actor's scale in 3D",
    },
    NodeSpec {
        id: "teleport_to",
        display_name: "Teleport Actor",
        category: NodeCategory::Actor,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("K2_TeleportTo"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::struct_in("DestLocation", VECTOR),
            PinSpec::struct_in("DestRotation", ROTATOR),
            PinSpec::bool_out("ReturnValue"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["teleport", "warp", "move", "actor"],
        tooltip: "Teleport the actor to a new location and rotation",
    },
    NodeSpec {
        id: "spawn_actor",
        display_name: "Spawn Actor from Class",
        category: NodeCategory::Actor,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("BeginSpawningActorFromClass"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::object_in("ActorClass", "/Script/Engine.Actor"),
            PinSpec::struct_in("SpawnTransform", TRANSFORM),
            PinSpec::bool_in("bNoCollisionFail"),
            PinSpec::object_out("ReturnValue", "/Script/Engine.Actor"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["spawn", "create", "instantiate", "actor", "new"],
        tooltip: "Spawn a new actor of the given class",
    },
    // === GAME MODE ===
    NodeSpec {
        id: "get_game_mode",
        display_name: "Get Game Mode",
        category: NodeCategory::GameMode,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("GetGameMode"),
        pins: &[
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::object_out("ReturnValue", "/Script/Engine.GameMode"),
        ],
        keywords: &["game mode", "gamemode", "game"],
        tooltip: "Get the current game mode",
    },
    NodeSpec {
        id: "get_player_controller",
        display_name: "Get Player Controller",
        category: NodeCategory::GameMode,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("GetPlayerController"),
        pins: &[
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::int_in("PlayerIndex"),
            PinSpec::object_out("ReturnValue", "/Script/Engine.PlayerController"),
        ],
        keywords: &["player", "controller", "input"],
        tooltip: "Get the player controller at the given index",
    },
    NodeSpec {
        id: "get_player_pawn",
        display_name: "Get Player Pawn",
        category: NodeCategory::GameMode,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("GetPlayerPawn"),
        pins: &[
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::int_in("PlayerIndex"),
            PinSpec::object_out("ReturnValue", "/Script/Engine.Pawn"),
        ],
        keywords: &["player", "pawn", "character"],
        tooltip: "Get the pawn controlled by the player at the given index",
    },
    NodeSpec {
        id: "get_player_character",
        display_name: "Get Player Character",
        category: NodeCategory::GameMode,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("GetPlayerCharacter"),
        pins: &[
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::int_in("PlayerIndex"),
            PinSpec::object_out("ReturnValue", "/Script/Engine.Character"),
        ],
        keywords: &["player", "character"],
        tooltip: "Get the character controlled by the player",
    },
    // === COMPONENT ===
    NodeSpec {
        id: "set_visibility",
        display_name: "Set Visibility",
        category: NodeCategory::Component,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.SceneComponent"),
        function_name: Some("K2_SetVisibility"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::bool_in("bNewVisibility"),
            PinSpec::bool_in("bPropagateToChildren"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["visibility", "visible", "show", "hide", "component"],
        tooltip: "Set whether this component is visible",
    },
    NodeSpec {
        id: "set_collision_enabled",
        display_name: "Set Collision Enabled",
        category: NodeCategory::Component,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PrimitiveComponent"),
        function_name: Some("SetCollisionEnabled"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::int_in("NewType"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["collision", "physics", "overlap", "block", "component"],
        tooltip: "Set the collision enabled mode for this component",
    },
    NodeSpec {
        id: "attach_component",
        display_name: "Attach Component to Component",
        category: NodeCategory::Component,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.SceneComponent"),
        function_name: Some("K2_AttachToComponent"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("Parent", "/Script/Engine.SceneComponent"),
            PinSpec::name_in("SocketName"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["attach", "parent", "component", "socket"],
        tooltip: "Attach this component to another component",
    },
    NodeSpec {
        id: "detach_from_component",
        display_name: "Detach from Component",
        category: NodeCategory::Component,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.SceneComponent"),
        function_name: Some("K2_DetachFromComponent"),
        pins: &[PinSpec::exec_in("execute"), PinSpec::exec_out("then")],
        keywords: &["detach", "remove", "component", "parent"],
        tooltip: "Detach this component from its parent",
    },
    // === PHYSICS ===
    NodeSpec {
        id: "set_simulate_physics",
        display_name: "Set Simulate Physics",
        category: NodeCategory::Physics,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PrimitiveComponent"),
        function_name: Some("SetSimulatePhysics"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::bool_in("bSimulate"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["physics", "simulate", "rigid body", "component"],
        tooltip: "Enable or disable physics simulation on this component",
    },
    NodeSpec {
        id: "add_impulse",
        display_name: "Add Impulse",
        category: NodeCategory::Physics,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PrimitiveComponent"),
        function_name: Some("AddImpulse"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::struct_in("Impulse", VECTOR),
            PinSpec::name_in("BoneName"),
            PinSpec::bool_in("bVelocityChange"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["impulse", "force", "physics", "push"],
        tooltip: "Add an impulse to this physics component",
    },
    NodeSpec {
        id: "add_force",
        display_name: "Add Force",
        category: NodeCategory::Physics,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PrimitiveComponent"),
        function_name: Some("AddForce"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::struct_in("Force", VECTOR),
            PinSpec::name_in("BoneName"),
            PinSpec::bool_in("bAccelChange"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["force", "physics", "push", "acceleration"],
        tooltip: "Apply a continuous force to this physics component",
    },
    // === INPUT ===
    NodeSpec {
        id: "is_input_key_down",
        display_name: "Is Input Key Down",
        category: NodeCategory::Input,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PlayerController"),
        function_name: Some("IsInputKeyDown"),
        pins: &[
            PinSpec::struct_in("Key", "/Script/InputCore.Key"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["input", "key", "down", "pressed", "button"],
        tooltip: "Return true if the given key is currently held down",
    },
    NodeSpec {
        id: "get_input_axis_value",
        display_name: "Get Input Axis Value",
        category: NodeCategory::Input,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PlayerController"),
        function_name: Some("GetInputAxisValue"),
        pins: &[
            PinSpec::name_in("InputAxisName"),
            PinSpec::float_out("ReturnValue"),
        ],
        keywords: &["input", "axis", "analog", "joystick", "value"],
        tooltip: "Get the current value of the named input axis",
    },
    NodeSpec {
        id: "enable_input",
        display_name: "Enable Input",
        category: NodeCategory::Input,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("EnableInput"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("PlayerController", "/Script/Engine.PlayerController"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["input", "enable", "actor"],
        tooltip: "Enable input processing for this actor",
    },
    NodeSpec {
        id: "disable_input",
        display_name: "Disable Input",
        category: NodeCategory::Input,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("DisableInput"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("PlayerController", "/Script/Engine.PlayerController"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["input", "disable", "actor"],
        tooltip: "Disable input processing for this actor",
    },
    NodeSpec {
        id: "event_on_clicked",
        display_name: "Event On Clicked",
        category: NodeCategory::Input,
        node_class: "/Script/BlueprintGraph.K2Node_ComponentBoundEvent",
        function_parent: None,
        function_name: Some("ReceiveActorOnClicked"),
        pins: &[
            PinSpec::object_out("TouchedComponent", "/Script/Engine.PrimitiveComponent"),
            PinSpec::struct_in("ButtonPressed", "/Script/InputCore.Key"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["click", "mouse", "input", "event", "pressed"],
        tooltip: "Called when the actor is clicked with the mouse",
    },
    // === NETWORK ===
    NodeSpec {
        id: "has_authority",
        display_name: "Has Authority",
        category: NodeCategory::Network,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.Actor"),
        function_name: Some("HasAuthority"),
        pins: &[PinSpec::bool_out("ReturnValue")],
        keywords: &["authority", "server", "network", "replicated"],
        tooltip: "Return true if this actor is on an authority (server) machine",
    },
    NodeSpec {
        id: "is_local_player_controller",
        display_name: "Is Local Player Controller",
        category: NodeCategory::Network,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PlayerController"),
        function_name: Some("IsLocalPlayerController"),
        pins: &[PinSpec::bool_out("ReturnValue")],
        keywords: &["local", "player", "controller", "network"],
        tooltip: "Return true if this is the local player controller",
    },
    // === ARRAY ===
    NodeSpec {
        id: "array_length",
        display_name: "Array Length",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_Length"),
        pins: &[
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["array", "length", "count", "size", "num"],
        tooltip: "Get the number of elements in an array",
    },
    NodeSpec {
        id: "array_get",
        display_name: "Array Get",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_Get"),
        pins: &[
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::int_in("Index"),
            PinSpec::object_out("Item", "/Script/CoreUObject.Object"),
        ],
        keywords: &["array", "get", "index", "element", "read"],
        tooltip: "Get the element at the specified array index",
    },
    NodeSpec {
        id: "array_set",
        display_name: "Array Set",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_Set"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::int_in("Index"),
            PinSpec::object_in("Item", "/Script/CoreUObject.Object"),
            PinSpec::bool_in("bSizeToFit"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["array", "set", "index", "element", "write"],
        tooltip: "Set the element at the specified array index",
    },
    NodeSpec {
        id: "array_add",
        display_name: "Array Add",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_Add"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::object_in("NewItem", "/Script/CoreUObject.Object"),
            PinSpec::int_out("ReturnValue"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["array", "add", "push", "append", "element"],
        tooltip: "Add an element to the end of an array",
    },
    NodeSpec {
        id: "array_remove",
        display_name: "Array Remove",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_RemoveItem"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::object_in("Item", "/Script/CoreUObject.Object"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["array", "remove", "delete", "element"],
        tooltip: "Remove all occurrences of an item from the array",
    },
    NodeSpec {
        id: "array_clear",
        display_name: "Array Clear",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_Clear"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["array", "clear", "empty", "reset"],
        tooltip: "Remove all elements from the array",
    },
    NodeSpec {
        id: "array_contains",
        display_name: "Array Contains",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_Contains"),
        pins: &[
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::object_in("ItemToFind", "/Script/CoreUObject.Object"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["array", "contains", "has", "find", "element"],
        tooltip: "Check if the array contains a specified element",
    },
    NodeSpec {
        id: "array_find",
        display_name: "Array Find",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_Find"),
        pins: &[
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::object_in("ItemToFind", "/Script/CoreUObject.Object"),
            PinSpec::int_out("ReturnValue"),
        ],
        keywords: &["array", "find", "search", "index", "element"],
        tooltip: "Find the index of the first occurrence of an element",
    },
    NodeSpec {
        id: "array_shuffle",
        display_name: "Array Shuffle",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_Shuffle"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["array", "shuffle", "random", "order"],
        tooltip: "Randomly shuffle the elements of the array",
    },
    NodeSpec {
        id: "array_resize",
        display_name: "Array Resize",
        category: NodeCategory::Array,
        node_class: CALL_FN,
        function_parent: Some(ARRAY_LIB),
        function_name: Some("Array_Resize"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("TargetArray", "/Script/CoreUObject.Object"),
            PinSpec::int_in("Size"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["array", "resize", "size", "set length"],
        tooltip: "Resize the array to the specified number of elements",
    },
    // === SAVE / LOAD ===
    NodeSpec {
        id: "create_save_game",
        display_name: "Create Save Game Object",
        category: NodeCategory::SaveLoad,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("CreateSaveGameObject"),
        pins: &[
            PinSpec::object_in("SaveGameClass", "/Script/Engine.SaveGame"),
            PinSpec::object_out("ReturnValue", "/Script/Engine.SaveGame"),
        ],
        keywords: &["save", "game", "create", "slot"],
        tooltip: "Create a new save game object of the given class",
    },
    NodeSpec {
        id: "save_game_to_slot",
        display_name: "Save Game to Slot",
        category: NodeCategory::SaveLoad,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("SaveGameToSlot"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("SaveGameObject", "/Script/Engine.SaveGame"),
            PinSpec::string_in("SlotName"),
            PinSpec::int_in("UserIndex"),
            PinSpec::bool_out("ReturnValue"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["save", "slot", "persist", "game", "write"],
        tooltip: "Save a save game object to the specified slot",
    },
    NodeSpec {
        id: "load_game_from_slot",
        display_name: "Load Game from Slot",
        category: NodeCategory::SaveLoad,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("LoadGameFromSlot"),
        pins: &[
            PinSpec::string_in("SlotName"),
            PinSpec::int_in("UserIndex"),
            PinSpec::object_out("ReturnValue", "/Script/Engine.SaveGame"),
        ],
        keywords: &["load", "slot", "read", "game", "save"],
        tooltip: "Load a save game object from the specified slot",
    },
    NodeSpec {
        id: "does_save_game_exist",
        display_name: "Does Save Game Exist",
        category: NodeCategory::SaveLoad,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("DoesSaveGameExist"),
        pins: &[
            PinSpec::string_in("SlotName"),
            PinSpec::int_in("UserIndex"),
            PinSpec::bool_out("ReturnValue"),
        ],
        keywords: &["save", "exists", "check", "slot", "game"],
        tooltip: "Check if a save game exists in the specified slot",
    },
    // === CAST ===
    NodeSpec {
        id: "cast_to_actor",
        display_name: "Cast to Actor",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_DynamicCast",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("Object", "/Script/CoreUObject.Object"),
            PinSpec::exec_out("CastSucceeded"),
            PinSpec::exec_out("CastFailed"),
            PinSpec::object_out("AsActor", "/Script/Engine.Actor"),
        ],
        keywords: &["cast", "actor", "convert"],
        tooltip: "Cast an object reference to Actor",
    },
    NodeSpec {
        id: "cast_to_character",
        display_name: "Cast to Character",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_DynamicCast",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("Object", "/Script/CoreUObject.Object"),
            PinSpec::exec_out("CastSucceeded"),
            PinSpec::exec_out("CastFailed"),
            PinSpec::object_out("AsCharacter", "/Script/Engine.Character"),
        ],
        keywords: &["cast", "character", "convert"],
        tooltip: "Cast an object reference to Character",
    },
    NodeSpec {
        id: "cast_to_player_controller",
        display_name: "Cast to Player Controller",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_DynamicCast",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("Object", "/Script/CoreUObject.Object"),
            PinSpec::exec_out("CastSucceeded"),
            PinSpec::exec_out("CastFailed"),
            PinSpec::object_out("AsPlayerController", "/Script/Engine.PlayerController"),
        ],
        keywords: &["cast", "player", "controller", "convert"],
        tooltip: "Cast an object reference to PlayerController",
    },
    NodeSpec {
        id: "cast_to_pawn",
        display_name: "Cast to Pawn",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_DynamicCast",
        function_parent: None,
        function_name: None,
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("Object", "/Script/CoreUObject.Object"),
            PinSpec::exec_out("CastSucceeded"),
            PinSpec::exec_out("CastFailed"),
            PinSpec::object_out("AsPawn", "/Script/Engine.Pawn"),
        ],
        keywords: &["cast", "pawn", "convert"],
        tooltip: "Cast an object reference to Pawn",
    },
    // === EVENTS ===
    NodeSpec {
        id: "event_begin_play",
        display_name: "Event Begin Play",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_Event",
        function_parent: None,
        function_name: Some("ReceiveBeginPlay"),
        pins: &[PinSpec::exec_out("then")],
        keywords: &["begin", "play", "start", "event", "init"],
        tooltip: "Called when the game starts or when spawned",
    },
    NodeSpec {
        id: "event_end_play",
        display_name: "Event End Play",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_Event",
        function_parent: None,
        function_name: Some("ReceiveEndPlay"),
        pins: &[PinSpec::int_out("EndPlayReason"), PinSpec::exec_out("then")],
        keywords: &["end", "play", "destroy", "event", "cleanup"],
        tooltip: "Called when the actor is being removed from play",
    },
    NodeSpec {
        id: "event_tick",
        display_name: "Event Tick",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_Event",
        function_parent: None,
        function_name: Some("ReceiveTick"),
        pins: &[
            PinSpec::float_out("DeltaSeconds"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["tick", "update", "frame", "event", "per frame"],
        tooltip: "Called every frame",
    },
    NodeSpec {
        id: "event_begin_overlap",
        display_name: "Event Begin Overlap",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_ComponentBoundEvent",
        function_parent: None,
        function_name: Some("ReceiveComponentBeginOverlap"),
        pins: &[
            PinSpec::object_out("OtherActor", "/Script/Engine.Actor"),
            PinSpec::object_out("OtherComp", "/Script/Engine.PrimitiveComponent"),
            PinSpec::struct_out("SweepResult", "/Script/Engine.HitResult"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["overlap", "trigger", "collision", "enter", "event"],
        tooltip: "Called when another actor begins overlapping this component",
    },
    NodeSpec {
        id: "event_end_overlap",
        display_name: "Event End Overlap",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_ComponentBoundEvent",
        function_parent: None,
        function_name: Some("ReceiveComponentEndOverlap"),
        pins: &[
            PinSpec::object_out("OtherActor", "/Script/Engine.Actor"),
            PinSpec::object_out("OtherComp", "/Script/Engine.PrimitiveComponent"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["overlap", "trigger", "collision", "exit", "event"],
        tooltip: "Called when another actor stops overlapping this component",
    },
    NodeSpec {
        id: "event_hit",
        display_name: "Event Hit",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_ComponentBoundEvent",
        function_parent: None,
        function_name: Some("ReceiveComponentHit"),
        pins: &[
            PinSpec::object_out("OtherActor", "/Script/Engine.Actor"),
            PinSpec::object_out("OtherComp", "/Script/Engine.PrimitiveComponent"),
            PinSpec::struct_out("NormalImpulse", VECTOR),
            PinSpec::struct_out("Hit", "/Script/Engine.HitResult"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["hit", "collision", "impact", "physics", "event"],
        tooltip: "Called when this component hits something solid",
    },
    NodeSpec {
        id: "event_destroyed",
        display_name: "Event Destroyed",
        category: NodeCategory::Utilities,
        node_class: "/Script/BlueprintGraph.K2Node_Event",
        function_parent: None,
        function_name: Some("ReceiveDestroyed"),
        pins: &[PinSpec::exec_out("then")],
        keywords: &["destroy", "event", "death", "remove", "delete"],
        tooltip: "Called when this actor is destroyed",
    },
    // === AUDIO ===
    NodeSpec {
        id: "play_sound_at_location",
        display_name: "Play Sound at Location",
        category: NodeCategory::Audio,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("PlaySoundAtLocation"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::object_in("Sound", "/Script/Engine.SoundBase"),
            PinSpec::struct_in("Location", VECTOR),
            PinSpec::struct_in("Rotation", ROTATOR),
            PinSpec::float_in("VolumeMultiplier"),
            PinSpec::float_in("PitchMultiplier"),
            PinSpec::float_in("StartTime"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["sound", "audio", "play", "sfx"],
        tooltip: "Play a sound effect at the specified world location",
    },
    NodeSpec {
        id: "play_sound_2d",
        display_name: "Play Sound 2D",
        category: NodeCategory::Audio,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("PlaySound2D"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("WorldContextObject", "/Script/CoreUObject.Object"),
            PinSpec::object_in("Sound", "/Script/Engine.SoundBase"),
            PinSpec::float_in("VolumeMultiplier"),
            PinSpec::float_in("PitchMultiplier"),
            PinSpec::float_in("StartTime"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["sound", "audio", "play", "2d", "ui", "music"],
        tooltip: "Play a non-spatialized (2D) sound",
    },
    // === AI ===
    NodeSpec {
        id: "simple_move_to_location",
        display_name: "Simple Move to Location",
        category: NodeCategory::AI,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("SimpleMoveToLocation"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("Controller", "/Script/Engine.Controller"),
            PinSpec::struct_in("Goal", VECTOR),
            PinSpec::exec_out("then"),
        ],
        keywords: &["ai", "move", "pathfind", "navigate", "location"],
        tooltip: "Move a pawn to the specified location using pathfinding",
    },
    NodeSpec {
        id: "simple_move_to_actor",
        display_name: "Simple Move to Actor",
        category: NodeCategory::AI,
        node_class: CALL_FN,
        function_parent: Some(GAME_LIB),
        function_name: Some("SimpleMoveToActor"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::object_in("Controller", "/Script/Engine.Controller"),
            PinSpec::object_in("Goal", "/Script/Engine.Actor"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["ai", "move", "pathfind", "navigate", "actor"],
        tooltip: "Move a pawn toward the specified actor using pathfinding",
    },
    // === RENDERING ===
    NodeSpec {
        id: "set_material",
        display_name: "Set Material",
        category: NodeCategory::Rendering,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PrimitiveComponent"),
        function_name: Some("SetMaterial"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::int_in("ElementIndex"),
            PinSpec::object_in("Material", "/Script/Engine.MaterialInterface"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["material", "render", "appearance", "texture", "component"],
        tooltip: "Set the material on a primitive component at the given slot index",
    },
    NodeSpec {
        id: "set_render_custom_depth",
        display_name: "Set Render Custom Depth",
        category: NodeCategory::Rendering,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PrimitiveComponent"),
        function_name: Some("SetRenderCustomDepth"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::bool_in("bValue"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["custom depth", "outline", "highlight", "render"],
        tooltip: "Enable or disable custom depth rendering",
    },
    NodeSpec {
        id: "set_custom_depth_stencil_value",
        display_name: "Set Custom Depth Stencil Value",
        category: NodeCategory::Rendering,
        node_class: CALL_FN,
        function_parent: Some("/Script/Engine.PrimitiveComponent"),
        function_name: Some("SetCustomDepthStencilValue"),
        pins: &[
            PinSpec::exec_in("execute"),
            PinSpec::int_in("Value"),
            PinSpec::exec_out("then"),
        ],
        keywords: &["stencil", "custom depth", "highlight", "render"],
        tooltip: "Set the custom depth stencil value for this component",
    },
    // === STRUCT ===
    NodeSpec {
        id: "make_transform",
        display_name: "Make Transform",
        category: NodeCategory::Struct,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("MakeTransform"),
        pins: &[
            PinSpec::struct_in("Location", VECTOR),
            PinSpec::struct_in("Rotation", ROTATOR),
            PinSpec::struct_in("Scale", VECTOR),
            PinSpec::struct_out("ReturnValue", TRANSFORM),
        ],
        keywords: &["make", "transform", "position", "rotation", "scale"],
        tooltip: "Construct a FTransform from location, rotation, and scale",
    },
    NodeSpec {
        id: "break_transform",
        display_name: "Break Transform",
        category: NodeCategory::Struct,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("BreakTransform"),
        pins: &[
            PinSpec::struct_in("InTransform", TRANSFORM),
            PinSpec::struct_out("Location", VECTOR),
            PinSpec::struct_out("Rotation", ROTATOR),
            PinSpec::struct_out("Scale", VECTOR),
        ],
        keywords: &[
            "break",
            "transform",
            "decompose",
            "position",
            "rotation",
            "scale",
        ],
        tooltip: "Decompose a FTransform into location, rotation, and scale",
    },
    NodeSpec {
        id: "make_rotator",
        display_name: "Make Rotator",
        category: NodeCategory::Struct,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("MakeRotator"),
        pins: &[
            PinSpec::float_in("Roll"),
            PinSpec::float_in("Pitch"),
            PinSpec::float_in("Yaw"),
            PinSpec::struct_out("ReturnValue", ROTATOR),
        ],
        keywords: &["make", "rotator", "rotation", "pitch", "yaw", "roll"],
        tooltip: "Construct a FRotator from roll, pitch, and yaw angles",
    },
    NodeSpec {
        id: "break_rotator",
        display_name: "Break Rotator",
        category: NodeCategory::Struct,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("BreakRotator"),
        pins: &[
            PinSpec::struct_in("InRot", ROTATOR),
            PinSpec::float_out("Roll"),
            PinSpec::float_out("Pitch"),
            PinSpec::float_out("Yaw"),
        ],
        keywords: &["break", "rotator", "rotation", "pitch", "yaw", "roll"],
        tooltip: "Decompose a FRotator into roll, pitch, and yaw angles",
    },
    NodeSpec {
        id: "make_linear_color",
        display_name: "Make Linear Color",
        category: NodeCategory::Struct,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("MakeColor"),
        pins: &[
            PinSpec::float_in("R"),
            PinSpec::float_in("G"),
            PinSpec::float_in("B"),
            PinSpec::float_in("A"),
            PinSpec::struct_out("ReturnValue", LINEAR_COLOR),
        ],
        keywords: &["make", "color", "linear color", "rgba"],
        tooltip: "Construct a FLinearColor from R, G, B, A components",
    },
    NodeSpec {
        id: "break_linear_color",
        display_name: "Break Linear Color",
        category: NodeCategory::Struct,
        node_class: CALL_FN,
        function_parent: Some(MATH_LIB),
        function_name: Some("BreakColor"),
        pins: &[
            PinSpec::struct_in("InColor", LINEAR_COLOR),
            PinSpec::float_out("R"),
            PinSpec::float_out("G"),
            PinSpec::float_out("B"),
            PinSpec::float_out("A"),
        ],
        keywords: &["break", "color", "linear color", "rgba"],
        tooltip: "Decompose a FLinearColor into R, G, B, A components",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> NodeRegistry {
        NodeRegistry::new()
    }

    #[test]
    fn registry_has_at_least_50_nodes() {
        let reg = registry();
        assert!(
            reg.len() >= 50,
            "Expected at least 50 nodes, got {}",
            reg.len()
        );
    }

    #[test]
    fn registry_has_at_least_100_nodes() {
        let reg = registry();
        assert!(
            reg.len() >= 100,
            "Expected at least 100 nodes, got {}",
            reg.len()
        );
    }

    #[test]
    fn search_print_returns_print_string() {
        let reg = registry();
        let results = reg.search("print");
        let ids: Vec<&str> = results.iter().map(|s| s.id).collect();
        assert!(
            ids.contains(&"print_string"),
            "search('print') should return print_string, got: {:?}",
            ids
        );
    }

    #[test]
    fn search_add_returns_multiple_math_nodes() {
        let reg = registry();
        let results = reg.search("add");
        assert!(
            results.len() >= 3,
            "search('add') should return at least 3 nodes, got {}",
            results.len()
        );
        let ids: Vec<&str> = results.iter().map(|s| s.id).collect();
        assert!(ids.contains(&"add_int"), "should include add_int");
        assert!(ids.contains(&"add_float"), "should include add_float");
    }

    #[test]
    fn by_category_math_returns_only_math_nodes() {
        let reg = registry();
        let nodes = reg.by_category(&NodeCategory::Math);
        assert!(!nodes.is_empty(), "Math category should have nodes");
        for node in &nodes {
            assert_eq!(
                node.category,
                NodeCategory::Math,
                "Node {} should be Math, got {:?}",
                node.id,
                node.category
            );
        }
    }

    #[test]
    fn create_add_int_returns_correct_bp_node() {
        let reg = registry();
        let node = reg
            .create("add_int", "Node_0")
            .expect("add_int should exist");
        assert_eq!(
            node.class, COMM_BIN,
            "add_int should use CommutativeAssociativeBinaryOperator"
        );
        assert_eq!(node.name, "Node_0");
        let fn_ref = node
            .properties
            .get("FunctionReference")
            .expect("should have FunctionReference");
        assert!(fn_ref.contains("KismetMathLibrary"));
        assert!(fn_ref.contains("Add_IntInt"));
    }

    #[test]
    fn create_nonexistent_returns_none() {
        let reg = registry();
        assert!(reg.create("nonexistent_node", "X").is_none());
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let reg = registry();
        assert!(reg.get("this_does_not_exist").is_none());
    }

    #[test]
    fn all_ids_is_sorted() {
        let reg = registry();
        let ids = reg.all_ids();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted, "all_ids() should be sorted alphabetically");
    }

    #[test]
    fn create_node_pins_are_correct() {
        let reg = registry();
        let node = reg
            .create("add_int", "AddNode")
            .expect("add_int should exist");
        assert_eq!(
            node.pins.len(),
            3,
            "add_int should have exactly 3 pins (A, B, ReturnValue)"
        );
        let a = node.find_pin("A").expect("should have pin A");
        assert!(
            matches!(a.direction, PinDirection::Input),
            "A should be an input pin"
        );
        assert_eq!(
            a.pin_type.category,
            PinCategory::Int,
            "A should be Int category"
        );
        assert_eq!(a.default_value.as_deref(), Some("0"));
        let ret = node
            .find_pin("ReturnValue")
            .expect("should have ReturnValue");
        assert!(
            matches!(ret.direction, PinDirection::Output),
            "ReturnValue should be an output pin"
        );
    }

    #[test]
    fn get_existing_node_returns_spec() {
        let reg = registry();
        let spec = reg.get("print_string").expect("print_string should exist");
        assert_eq!(spec.id, "print_string");
        assert_eq!(spec.category, NodeCategory::Debug);
    }

    #[test]
    fn by_category_debug_includes_print_string() {
        let reg = registry();
        let debug_nodes = reg.by_category(&NodeCategory::Debug);
        let ids: Vec<&str> = debug_nodes.iter().map(|s| s.id).collect();
        assert!(
            ids.contains(&"print_string"),
            "Debug category should contain print_string"
        );
    }

    #[test]
    fn search_is_case_insensitive() {
        let reg = registry();
        let lower = reg.search("print");
        let upper = reg.search("PRINT");
        let lower_ids: Vec<&str> = lower.iter().map(|s| s.id).collect();
        let upper_ids: Vec<&str> = upper.iter().map(|s| s.id).collect();
        assert_eq!(lower_ids, upper_ids, "search should be case-insensitive");
    }

    #[test]
    fn all_nodes_have_unique_ids() {
        let ids: Vec<&str> = ALL_NODES.iter().map(|n| n.id).collect();
        let mut seen = std::collections::HashSet::new();
        for id in &ids {
            assert!(seen.insert(id), "Duplicate node id: {}", id);
        }
    }

    #[test]
    fn default_registry_equals_new() {
        let r1 = NodeRegistry::new();
        let r2 = NodeRegistry::default();
        assert_eq!(r1.len(), r2.len());
    }
}
