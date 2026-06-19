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
