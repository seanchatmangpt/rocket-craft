//! JSON serializer — converts a Blueprint AST to a pretty-printed JSON string.

use crate::ast::Blueprint;

pub struct JsonSerializer;

impl JsonSerializer {
    /// Serialize a `Blueprint` to a pretty-printed JSON string.
    pub fn serialize(bp: &Blueprint) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(bp)
    }

    /// Deserialize a `Blueprint` from a JSON string.
    pub fn deserialize(json: &str) -> Result<Blueprint, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Blueprint;

    fn make_bp() -> Blueprint {
        Blueprint::new("TestWidget", "UserWidget")
    }

    #[test]
    fn serialize_produces_valid_json() {
        let bp = make_bp();
        let json = JsonSerializer::serialize(&bp).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["name"], "TestWidget");
    }

    #[test]
    fn deserialize_roundtrips() {
        let bp = make_bp();
        let json = JsonSerializer::serialize(&bp).unwrap();
        let restored = JsonSerializer::deserialize(&json).unwrap();
        assert_eq!(restored.name, bp.name);
        assert_eq!(restored.parent_class, bp.parent_class);
    }

    #[test]
    fn deserialize_errors_on_invalid_json() {
        assert!(JsonSerializer::deserialize("{ bad json }").is_err());
    }

    #[test]
    fn serialize_empty_variables() {
        let bp = make_bp();
        let json = JsonSerializer::serialize(&bp).unwrap();
        assert!(json.contains("\"variables\": []"));
    }
}
