use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FfiValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<FfiValue>),
    Object(HashMap<String, FfiValue>),
}

impl FfiValue {
    pub fn as_str(&self) -> Option<&str> {
        if let FfiValue::Str(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let FfiValue::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let FfiValue::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "null".to_string())
    }

    pub fn from_json(s: &str) -> Result<Self, FfiError> {
        serde_json::from_str(s).map_err(|e| FfiError::new("PARSE_ERROR", e.to_string()))
    }

    pub fn null() -> Self {
        Self::Null
    }
}

impl From<String> for FfiValue {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<bool> for FfiValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<i64> for FfiValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<f64> for FfiValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, serde::Deserialize)]
#[error("FFI error [{code}]: {message}")]
pub struct FfiError {
    pub code: String,
    pub message: String,
}

impl FfiError {
    pub fn new(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: msg.into(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new("INTERNAL", msg)
    }

    pub fn invalid_arg(msg: impl Into<String>) -> Self {
        Self::new("INVALID_ARG", msg)
    }
}

pub type FfiResult<T> = Result<T, FfiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_value_from_string() {
        let v = FfiValue::from("hello".to_string());
        assert!(matches!(v, FfiValue::Str(_)));
        assert_eq!(v.as_str(), Some("hello"));
    }

    #[test]
    fn test_ffi_value_from_bool() {
        let v = FfiValue::from(true);
        assert!(matches!(v, FfiValue::Bool(true)));
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_ffi_value_from_i64() {
        let v = FfiValue::from(42i64);
        assert!(matches!(v, FfiValue::Int(42)));
        assert_eq!(v.as_int(), Some(42));
    }

    #[test]
    fn test_ffi_value_from_f64() {
        let v = FfiValue::from(3.14f64);
        assert!(matches!(v, FfiValue::Float(_)));
    }

    #[test]
    fn test_ffi_value_as_str_wrong_variant() {
        let v = FfiValue::Bool(true);
        assert_eq!(v.as_str(), None);
    }

    #[test]
    fn test_ffi_value_as_bool_wrong_variant() {
        let v = FfiValue::Str("true".to_string());
        assert_eq!(v.as_bool(), None);
    }

    #[test]
    fn test_ffi_value_as_int_wrong_variant() {
        let v = FfiValue::Str("42".to_string());
        assert_eq!(v.as_int(), None);
    }

    #[test]
    fn test_ffi_value_to_json_from_json_roundtrip() {
        let original = FfiValue::Str("hello".to_string());
        let json = original.to_json();
        let recovered = FfiValue::from_json(&json).unwrap();
        assert_eq!(recovered.as_str(), Some("hello"));
    }

    #[test]
    fn test_ffi_value_from_json_invalid() {
        let result = FfiValue::from_json("not valid json{{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_ffi_error_new() {
        let e = FfiError::new("MY_CODE", "my message");
        assert_eq!(e.code, "MY_CODE");
        assert_eq!(e.message, "my message");
    }

    #[test]
    fn test_ffi_error_internal() {
        let e = FfiError::internal("something broke");
        assert_eq!(e.code, "INTERNAL");
        assert!(e.message.contains("something broke"));
    }

    #[test]
    fn test_ffi_error_invalid_arg() {
        let e = FfiError::invalid_arg("bad param");
        assert_eq!(e.code, "INVALID_ARG");
        assert!(e.message.contains("bad param"));
    }

    #[test]
    fn test_ffi_value_null() {
        let v = FfiValue::null();
        assert!(matches!(v, FfiValue::Null));
    }
}
