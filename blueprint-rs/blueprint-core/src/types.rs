use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A UE4-style GUID, stored as uppercase hex with no dashes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UeGuid(pub String);

impl UeGuid {
    pub fn new() -> Self {
        let id = Uuid::new_v4();
        let s = id.as_simple().to_string().to_uppercase();
        UeGuid(s)
    }
}

impl Default for UeGuid {
    fn default() -> Self {
        UeGuid::new()
    }
}

impl std::str::FromStr for UeGuid {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UeGuid(s.to_string()))
    }
}

impl std::fmt::Display for UeGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Blueprint pin categories — matches UE4 K2 pin categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PinCategory {
    Exec,
    Boolean,
    Byte,
    Int,
    Int64,
    Float,
    Double,
    Name,
    String,
    Text,
    Object,
    Class,
    Interface,
    Struct,
    Enum,
    Delegate,
    SoftObject,
    SoftClass,
    Wildcard,
}

impl PinCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PinCategory::Exec => "exec",
            PinCategory::Boolean => "bool",
            PinCategory::Byte => "byte",
            PinCategory::Int => "int",
            PinCategory::Int64 => "int64",
            PinCategory::Float => "float",
            PinCategory::Double => "double",
            PinCategory::Name => "name",
            PinCategory::String => "string",
            PinCategory::Text => "text",
            PinCategory::Object => "object",
            PinCategory::Class => "class",
            PinCategory::Interface => "interface",
            PinCategory::Struct => "struct",
            PinCategory::Enum => "byte",
            PinCategory::Delegate => "delegate",
            PinCategory::SoftObject => "softobject",
            PinCategory::SoftClass => "softclass",
            PinCategory::Wildcard => "wildcard",
        }
    }
}

/// Container type for Blueprint pins
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContainerType {
    None,
    Array,
    Set,
    Map,
}

/// Direction of a pin (input or output)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PinDirection {
    Input,
    Output,
}

/// Full type information for a Blueprint pin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinType {
    pub category: PinCategory,
    pub sub_category: Option<String>,
    pub sub_category_object: Option<String>,
    pub container: ContainerType,
    pub is_reference: bool,
    pub is_const: bool,
}

impl PinType {
    pub fn new(category: PinCategory) -> Self {
        Self {
            category,
            sub_category: None,
            sub_category_object: None,
            container: ContainerType::None,
            is_reference: false,
            is_const: false,
        }
    }

    pub fn exec() -> Self {
        Self::new(PinCategory::Exec)
    }

    pub fn bool() -> Self {
        Self::new(PinCategory::Boolean)
    }

    pub fn byte() -> Self {
        Self::new(PinCategory::Byte)
    }

    pub fn int() -> Self {
        Self::new(PinCategory::Int)
    }

    pub fn int64() -> Self {
        Self::new(PinCategory::Int64)
    }

    pub fn float() -> Self {
        Self::new(PinCategory::Float)
    }

    pub fn double() -> Self {
        Self::new(PinCategory::Double)
    }

    pub fn name() -> Self {
        Self::new(PinCategory::Name)
    }

    pub fn string() -> Self {
        Self::new(PinCategory::String)
    }

    pub fn text() -> Self {
        Self::new(PinCategory::Text)
    }

    pub fn object(class_path: impl Into<String>) -> Self {
        Self {
            category: PinCategory::Object,
            sub_category: None,
            sub_category_object: Some(class_path.into()),
            container: ContainerType::None,
            is_reference: false,
            is_const: false,
        }
    }

    pub fn class(class_path: impl Into<String>) -> Self {
        Self {
            category: PinCategory::Class,
            sub_category: None,
            sub_category_object: Some(class_path.into()),
            container: ContainerType::None,
            is_reference: false,
            is_const: false,
        }
    }

    pub fn struct_type(struct_path: impl Into<String>) -> Self {
        Self {
            category: PinCategory::Struct,
            sub_category: None,
            sub_category_object: Some(struct_path.into()),
            container: ContainerType::None,
            is_reference: false,
            is_const: false,
        }
    }

    pub fn wildcard() -> Self {
        Self::new(PinCategory::Wildcard)
    }

    pub fn as_array(mut self) -> Self {
        self.container = ContainerType::Array;
        self
    }

    pub fn as_ref(mut self) -> Self {
        self.is_reference = true;
        self
    }

    pub fn as_const(mut self) -> Self {
        self.is_const = true;
        self
    }
}

/// 2D position of a node in the Blueprint graph editor
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodePos {
    pub x: i32,
    pub y: i32,
}

impl NodePos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── UeGuid ────────────────────────────────────────────────────────────────

    #[test]
    fn ueguid_new_is_32_uppercase_hex_chars() {
        let g = UeGuid::new();
        assert_eq!(g.0.len(), 32);
        assert!(g.0.chars().all(|c| c.is_ascii_hexdigit() && !c.is_lowercase()));
    }

    #[test]
    fn ueguid_two_news_are_distinct() {
        let a = UeGuid::new();
        let b = UeGuid::new();
        assert_ne!(a, b);
    }

    #[test]
    fn ueguid_display_returns_inner_string() {
        let g = UeGuid("ABCDEF1234567890ABCDEF1234567890".into());
        assert_eq!(g.to_string(), "ABCDEF1234567890ABCDEF1234567890");
    }

    #[test]
    fn ueguid_from_str_roundtrip() {
        use std::str::FromStr;
        let g: UeGuid = UeGuid::from_str("DEADBEEF").unwrap();
        assert_eq!(g.0, "DEADBEEF");
    }

    // ── PinCategory::as_str ───────────────────────────────────────────────────

    #[test]
    fn pin_category_exec_as_str() {
        assert_eq!(PinCategory::Exec.as_str(), "exec");
    }

    #[test]
    fn pin_category_float_as_str() {
        assert_eq!(PinCategory::Float.as_str(), "float");
    }

    #[test]
    fn pin_category_enum_maps_to_byte() {
        // UE4 enums are stored as byte pins
        assert_eq!(PinCategory::Enum.as_str(), "byte");
    }

    #[test]
    fn pin_category_wildcard_as_str() {
        assert_eq!(PinCategory::Wildcard.as_str(), "wildcard");
    }

    // ── PinType constructors ──────────────────────────────────────────────────

    #[test]
    fn pin_type_exec_has_exec_category() {
        let p = PinType::exec();
        assert_eq!(p.category, PinCategory::Exec);
    }

    #[test]
    fn pin_type_bool_category() {
        assert_eq!(PinType::bool().category, PinCategory::Boolean);
    }

    #[test]
    fn pin_type_int_category() {
        assert_eq!(PinType::int().category, PinCategory::Int);
    }

    #[test]
    fn pin_type_float_category() {
        assert_eq!(PinType::float().category, PinCategory::Float);
    }

    #[test]
    fn pin_type_string_category() {
        assert_eq!(PinType::string().category, PinCategory::String);
    }

    #[test]
    fn pin_type_object_stores_class_path() {
        let p = PinType::object("Engine.Actor");
        assert_eq!(p.category, PinCategory::Object);
        assert_eq!(p.sub_category_object.as_deref(), Some("Engine.Actor"));
    }

    #[test]
    fn pin_type_as_array_sets_container() {
        let p = PinType::int().as_array();
        assert_eq!(p.container, ContainerType::Array);
    }

    #[test]
    fn pin_type_as_ref_sets_is_ref() {
        let p = PinType::bool().as_ref();
        assert!(p.is_reference);
    }

    #[test]
    fn pin_type_as_const_sets_is_const() {
        let p = PinType::float().as_const();
        assert!(p.is_const);
    }

    #[test]
    fn pin_type_new_defaults_to_none_container() {
        let p = PinType::new(PinCategory::String);
        assert_eq!(p.container, ContainerType::None);
        assert!(!p.is_reference);
        assert!(!p.is_const);
    }

    // ── ContainerType / PinDirection ──────────────────────────────────────────

    #[test]
    fn container_types_are_distinct() {
        assert_ne!(ContainerType::None, ContainerType::Array);
        assert_ne!(ContainerType::Array, ContainerType::Set);
        assert_ne!(ContainerType::Set, ContainerType::Map);
    }

    #[test]
    fn pin_directions_are_distinct() {
        assert_ne!(PinDirection::Input, PinDirection::Output);
    }
}
