use crate::types::{FfiError, FfiResult, FfiValue};
use std::collections::HashMap;

pub trait ToFfi {
    fn to_ffi(&self) -> FfiValue;
}

pub trait FromFfi: Sized {
    fn from_ffi(v: &FfiValue) -> FfiResult<Self>;
}

impl ToFfi for String {
    fn to_ffi(&self) -> FfiValue {
        FfiValue::Str(self.clone())
    }
}

impl ToFfi for bool {
    fn to_ffi(&self) -> FfiValue {
        FfiValue::Bool(*self)
    }
}

impl ToFfi for i64 {
    fn to_ffi(&self) -> FfiValue {
        FfiValue::Int(*self)
    }
}

impl ToFfi for f64 {
    fn to_ffi(&self) -> FfiValue {
        FfiValue::Float(*self)
    }
}

impl FromFfi for String {
    fn from_ffi(v: &FfiValue) -> FfiResult<Self> {
        v.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| FfiError::invalid_arg("expected string"))
    }
}

impl FromFfi for bool {
    fn from_ffi(v: &FfiValue) -> FfiResult<Self> {
        v.as_bool().ok_or_else(|| FfiError::invalid_arg("expected bool"))
    }
}

/// Convert a `serde_json::Value` into a `FfiValue`.
pub fn json_to_ffi(v: &serde_json::Value) -> FfiValue {
    match v {
        serde_json::Value::Null => FfiValue::Null,
        serde_json::Value::Bool(b) => FfiValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                FfiValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                FfiValue::Float(f)
            } else {
                FfiValue::Null
            }
        }
        serde_json::Value::String(s) => FfiValue::Str(s.clone()),
        serde_json::Value::Array(arr) => {
            FfiValue::Array(arr.iter().map(json_to_ffi).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, val) in obj {
                map.insert(k.clone(), json_to_ffi(val));
            }
            FfiValue::Object(map)
        }
    }
}

/// Convert a `FfiValue` into a `serde_json::Value`.
pub fn ffi_to_json(v: &FfiValue) -> serde_json::Value {
    match v {
        FfiValue::Null => serde_json::Value::Null,
        FfiValue::Bool(b) => serde_json::Value::Bool(*b),
        FfiValue::Int(i) => serde_json::Value::Number((*i).into()),
        FfiValue::Float(f) => serde_json::json!(*f),
        FfiValue::Str(s) => serde_json::Value::String(s.clone()),
        FfiValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(ffi_to_json).collect())
        }
        FfiValue::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (k, val) in obj {
                map.insert(k.clone(), ffi_to_json(val));
            }
            serde_json::Value::Object(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_ffi_string() {
        let s = "hello".to_string();
        let v = s.to_ffi();
        assert_eq!(v.as_str(), Some("hello"));
    }

    #[test]
    fn test_to_ffi_bool() {
        let b = true;
        let v = b.to_ffi();
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_to_ffi_i64() {
        let i: i64 = 99;
        let v = i.to_ffi();
        assert_eq!(v.as_int(), Some(99));
    }

    #[test]
    fn test_to_ffi_f64() {
        let f: f64 = 2.71;
        let v = f.to_ffi();
        assert!(matches!(v, FfiValue::Float(_)));
    }

    #[test]
    fn test_from_ffi_string_success() {
        let v = FfiValue::Str("world".to_string());
        let s = String::from_ffi(&v).unwrap();
        assert_eq!(s, "world");
    }

    #[test]
    fn test_from_ffi_string_from_bool_fails() {
        let v = FfiValue::Bool(false);
        let result = String::from_ffi(&v);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "INVALID_ARG");
    }

    #[test]
    fn test_from_ffi_bool_success() {
        let v = FfiValue::Bool(true);
        let b = bool::from_ffi(&v).unwrap();
        assert!(b);
    }

    #[test]
    fn test_json_to_ffi_and_back_roundtrip() {
        let json = serde_json::json!({
            "name": "test",
            "value": 42,
            "flag": true,
            "nested": { "x": 1.5 }
        });
        let ffi = json_to_ffi(&json);
        let back = ffi_to_json(&ffi);
        assert_eq!(json["name"], back["name"]);
        assert_eq!(json["value"], back["value"]);
        assert_eq!(json["flag"], back["flag"]);
    }

    #[test]
    fn test_json_to_ffi_array() {
        let json = serde_json::json!([1, 2, 3]);
        let ffi = json_to_ffi(&json);
        assert!(matches!(ffi, FfiValue::Array(_)));
        let back = ffi_to_json(&ffi);
        assert_eq!(back, json);
    }

    #[test]
    fn test_json_to_ffi_null() {
        let json = serde_json::Value::Null;
        let ffi = json_to_ffi(&json);
        assert!(matches!(ffi, FfiValue::Null));
    }
}
