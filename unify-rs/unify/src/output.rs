use serde_json::Value;

/// Output from a unify command, carrying data, success status, and an optional message.
#[derive(Debug, Clone)]
pub struct Output {
    pub data: Value,
    pub success: bool,
    pub message: Option<String>,
}

impl Output {
    /// Construct a successful output with the given JSON data.
    pub fn ok(data: Value) -> Self {
        Self {
            data,
            success: true,
            message: None,
        }
    }

    /// Construct a successful output with a plain string message and null data.
    pub fn success_msg(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        Self {
            data: Value::Null,
            success: true,
            message: Some(msg),
        }
    }

    /// Construct a failed output with an error message.
    pub fn error(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        Self {
            data: Value::Null,
            success: false,
            message: Some(msg),
        }
    }

    /// Serialize to compact JSON.
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "success": self.success,
            "message": self.message,
            "data": self.data,
        })
        .to_string()
    }

    /// Pretty-print for a human-readable terminal.
    pub fn to_human(&self) -> String {
        let status = if self.success { "OK" } else { "ERROR" };
        let mut parts = vec![format!("[{}]", status)];
        if let Some(msg) = &self.message {
            parts.push(msg.clone());
        }
        if self.data != Value::Null {
            parts.push(serde_json::to_string_pretty(&self.data).unwrap_or_default());
        }
        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ok_creates_successful_output_with_data() {
        let o = Output::ok(json!({"key": "value"}));
        assert!(o.success);
        assert!(o.message.is_none());
        assert_eq!(o.data, json!({"key": "value"}));
    }

    #[test]
    fn success_msg_has_null_data_and_message() {
        let o = Output::success_msg("all done");
        assert!(o.success);
        assert_eq!(o.message.as_deref(), Some("all done"));
        assert_eq!(o.data, Value::Null);
    }

    #[test]
    fn error_creates_failed_output_with_message() {
        let o = Output::error("something went wrong");
        assert!(!o.success);
        assert_eq!(o.message.as_deref(), Some("something went wrong"));
    }

    #[test]
    fn to_json_contains_success_and_message_fields() {
        let o = Output::success_msg("done");
        let json = o.to_json();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("done"));
    }

    #[test]
    fn to_human_ok_prefix() {
        let o = Output::ok(json!(null));
        assert!(o.to_human().starts_with("[OK]"));
    }

    #[test]
    fn to_human_error_prefix() {
        let o = Output::error("bad");
        assert!(o.to_human().starts_with("[ERROR]"));
        assert!(o.to_human().contains("bad"));
    }

    #[test]
    fn to_human_includes_data_when_not_null() {
        let o = Output::ok(json!({"x": 1}));
        let human = o.to_human();
        assert!(human.contains("\"x\""));
    }
}
