use crate::types::{FfiError, FfiResult, FfiValue};
use std::collections::HashMap;

pub type FfiHandler = Box<dyn Fn(FfiValue) -> FfiResult<FfiValue> + Send + Sync>;

pub struct FfiCommandRegistry {
    handlers: HashMap<String, FfiHandler>,
}

impl FfiCommandRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        name: impl Into<String>,
        handler: impl Fn(FfiValue) -> FfiResult<FfiValue> + Send + Sync + 'static,
    ) {
        self.handlers.insert(name.into(), Box::new(handler));
    }

    pub fn call(&self, name: &str, input: FfiValue) -> FfiResult<FfiValue> {
        match self.handlers.get(name) {
            Some(handler) => handler(input),
            None => Err(FfiError::new(
                "NOT_FOUND",
                format!("command '{}' not registered", name),
            )),
        }
    }

    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.handlers.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    pub fn has(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// Registers built-in commands: "version", "echo", "ping".
    pub fn with_builtins(mut self) -> Self {
        self.register("version", |_input| {
            Ok(FfiValue::Str(env!("CARGO_PKG_VERSION").to_string()))
        });

        self.register("echo", |input| Ok(input));

        self.register("ping", |_input| Ok(FfiValue::Str("pong".to_string())));

        self
    }
}

impl Default for FfiCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let reg = FfiCommandRegistry::new();
        assert!(reg.list().is_empty());
    }

    #[test]
    fn test_registry_register_and_has() {
        let mut reg = FfiCommandRegistry::new();
        reg.register("my_cmd", |_| Ok(FfiValue::Null));
        assert!(reg.has("my_cmd"));
        assert!(!reg.has("other"));
    }

    #[test]
    fn test_registry_list() {
        let mut reg = FfiCommandRegistry::new();
        reg.register("b_cmd", |_| Ok(FfiValue::Null));
        reg.register("a_cmd", |_| Ok(FfiValue::Null));
        let list = reg.list();
        assert_eq!(list.len(), 2);
        // Sorted
        assert_eq!(list[0], "a_cmd");
        assert_eq!(list[1], "b_cmd");
    }

    #[test]
    fn test_registry_call_success() {
        let mut reg = FfiCommandRegistry::new();
        reg.register("greet", |_| Ok(FfiValue::Str("hello".to_string())));
        let result = reg.call("greet", FfiValue::Null).unwrap();
        assert_eq!(result.as_str(), Some("hello"));
    }

    #[test]
    fn test_registry_call_unknown_returns_err() {
        let reg = FfiCommandRegistry::new();
        let result = reg.call("nonexistent", FfiValue::Null);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "NOT_FOUND");
    }

    #[test]
    fn test_with_builtins_version() {
        let reg = FfiCommandRegistry::new().with_builtins();
        let result = reg.call("version", FfiValue::Null);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert!(v.as_str().is_some());
    }

    #[test]
    fn test_with_builtins_echo() {
        let reg = FfiCommandRegistry::new().with_builtins();
        let input = FfiValue::Str("test-payload".to_string());
        let result = reg.call("echo", input).unwrap();
        assert_eq!(result.as_str(), Some("test-payload"));
    }

    #[test]
    fn test_with_builtins_ping() {
        let reg = FfiCommandRegistry::new().with_builtins();
        let result = reg.call("ping", FfiValue::Null).unwrap();
        assert_eq!(result.as_str(), Some("pong"));
    }

    #[test]
    fn test_with_builtins_has_all() {
        let reg = FfiCommandRegistry::new().with_builtins();
        assert!(reg.has("version"));
        assert!(reg.has("echo"));
        assert!(reg.has("ping"));
    }
}
