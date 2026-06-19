use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub file_path: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub line: usize,
    pub column: usize,
    pub kind: String,
    pub construct: String,
    pub context: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Observation {
        Observation {
            file_path: "src/foo.rs".into(),
            start_byte: 10,
            end_byte: 20,
            line: 3,
            column: 5,
            kind: "cargo_lock".into(),
            construct: "tower-lsp".into(),
            context: "tower-lsp = { version = \"0.20\" }".into(),
            message: "Forbidden dependency found".into(),
        }
    }

    #[test]
    fn observation_serializes_to_json() {
        let obs = sample();
        let json = serde_json::to_string(&obs).expect("serialize");
        assert!(json.contains("\"file_path\""));
        assert!(json.contains("src/foo.rs"));
    }

    #[test]
    fn observation_round_trips_json() {
        let obs = sample();
        let json = serde_json::to_string(&obs).unwrap();
        let back: Observation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.file_path, obs.file_path);
        assert_eq!(back.line, obs.line);
        assert_eq!(back.construct, obs.construct);
    }

    #[test]
    fn observation_clone_is_independent() {
        let obs = sample();
        let mut clone = obs.clone();
        clone.file_path = "other.rs".into();
        assert_eq!(obs.file_path, "src/foo.rs");
    }

    #[test]
    fn observation_byte_range_preserved() {
        let obs = sample();
        assert_eq!(obs.start_byte, 10);
        assert_eq!(obs.end_byte, 20);
        assert!(obs.end_byte > obs.start_byte);
    }
}
