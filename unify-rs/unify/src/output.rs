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
