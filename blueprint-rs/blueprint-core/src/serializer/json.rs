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
