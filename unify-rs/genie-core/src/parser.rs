use crate::errors::GenieError;
use crate::spec::{
    Actor, Bounds3D, Object, Place, Relationship, RelationshipType, Rule, RuleSeverity, Vector3,
    WorldSpec,
};
use regex::Regex;

/// Parser for the Genie 26 natural language intent commands using Regex.
pub struct IntentParser;

impl IntentParser {
    /// Parses a multi-line intent string into a WorldSpec.
    pub fn parse(intent: &str) -> Result<WorldSpec, GenieError> {
        let mut spec = WorldSpec::new();

        // Regex definitions
        let place_re = Regex::new(
            r#"^create\s+place\s+(\S+)\s+name\s+((?:"[^"]*")|(?:'[^']*')|(?:\S+))\s+at\s*\(\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*\)\s+bounds\s*\(\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*\)$"#
        ).map_err(|e| GenieError::Parse(format!("Regex compile error: {}", e)))?;

        let actor_re = Regex::new(
            r#"^create\s+actor\s+(\S+)\s+name\s+((?:"[^"]*")|(?:'[^']*')|(?:\S+))\s+role\s+(\S+)\s+in\s+(\S+)$"#
        ).map_err(|e| GenieError::Parse(format!("Regex compile error: {}", e)))?;

        let object_re = Regex::new(
            r#"^create\s+object\s+(\S+)\s+name\s+((?:"[^"]*")|(?:'[^']*')|(?:\S+))\s+class\s+(\S+)\s+in\s+(\S+)$"#
        ).map_err(|e| GenieError::Parse(format!("Regex compile error: {}", e)))?;

        let relationship_re =
            Regex::new(r"^create\s+relationship\s+(\S+)\s+(\S+)\s+from\s+(\S+)\s+to\s+(\S+)$")
                .map_err(|e| GenieError::Parse(format!("Regex compile error: {}", e)))?;

        let rule_re = Regex::new(
            r#"^create\s+rule\s+(\S+)\s+name\s+((?:"[^"]*")|(?:'[^']*')|(?:\S+))\s+expression\s+((?:"[^"]*")|(?:'[^']*')|(?:\S+))\s+severity\s+(\S+)$"#
        ).map_err(|e| GenieError::Parse(format!("Regex compile error: {}", e)))?;

        for (idx, line) in intent.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if place_re.is_match(trimmed) {
                let caps = place_re.captures(trimmed).unwrap();
                let id = caps[1].to_string();
                let name = strip_quotes(&caps[2]);

                let x = caps[3].parse::<f32>().map_err(|e| {
                    GenieError::Parse(format!("Line {}: invalid x: {}", line_num, e))
                })?;
                let y = caps[4].parse::<f32>().map_err(|e| {
                    GenieError::Parse(format!("Line {}: invalid y: {}", line_num, e))
                })?;
                let z = caps[5].parse::<f32>().map_err(|e| {
                    GenieError::Parse(format!("Line {}: invalid z: {}", line_num, e))
                })?;

                let w = caps[6].parse::<f32>().map_err(|e| {
                    GenieError::Parse(format!("Line {}: invalid width: {}", line_num, e))
                })?;
                let l = caps[7].parse::<f32>().map_err(|e| {
                    GenieError::Parse(format!("Line {}: invalid length: {}", line_num, e))
                })?;
                let h = caps[8].parse::<f32>().map_err(|e| {
                    GenieError::Parse(format!("Line {}: invalid height: {}", line_num, e))
                })?;

                let bounds = Bounds3D::new(Vector3::new(x, y, z), Vector3::new(w, l, h));
                spec.places.push(Place::new(id, name, bounds));
            } else if actor_re.is_match(trimmed) {
                let caps = actor_re.captures(trimmed).unwrap();
                let id = caps[1].to_string();
                let name = strip_quotes(&caps[2]);
                let role = caps[3].to_string();
                let place_id = caps[4].to_string();

                spec.actors.push(Actor::new(id, name, role, place_id));
            } else if object_re.is_match(trimmed) {
                let caps = object_re.captures(trimmed).unwrap();
                let id = caps[1].to_string();
                let name = strip_quotes(&caps[2]);
                let class = caps[3].to_string();
                let place_id = caps[4].to_string();

                spec.objects.push(Object::new(id, name, class, place_id));
            } else if relationship_re.is_match(trimmed) {
                let caps = relationship_re.captures(trimmed).unwrap();
                let id = caps[1].to_string();
                let rel_type_str = caps[2].to_string();
                let source = caps[3].to_string();
                let target = caps[4].to_string();

                let rel_type = match rel_type_str.to_lowercase().as_str() {
                    "connects" => RelationshipType::Connects,
                    "contains" => RelationshipType::Contains,
                    "owns" => RelationshipType::Owns,
                    "adjacent_to" => RelationshipType::AdjacentTo,
                    "controls" => RelationshipType::Controls,
                    other => RelationshipType::Custom(other.to_string()),
                };

                spec.relationships
                    .push(Relationship::new(id, rel_type, source, target));
            } else if rule_re.is_match(trimmed) {
                let caps = rule_re.captures(trimmed).unwrap();
                let id = caps[1].to_string();
                let name = strip_quotes(&caps[2]);
                let expression = strip_quotes(&caps[3]);
                let severity_str = caps[4].to_string();

                let severity = match severity_str.to_lowercase().as_str() {
                    "info" => RuleSeverity::Info,
                    "warning" => RuleSeverity::Warning,
                    "error" => RuleSeverity::Error,
                    other => {
                        return Err(GenieError::Parse(format!(
                            "Line {}: unknown rule severity: {}",
                            line_num, other
                        )))
                    }
                };

                spec.rules.push(Rule::new(id, name, expression, severity));
            } else {
                return Err(GenieError::Parse(format!(
                    "Line {}: Command did not match any known pattern: '{}'",
                    line_num, trimmed
                )));
            }
        }

        Ok(spec)
    }
}

/// Helper function to strip surrounding double or single quotes from a parsed string value.
fn strip_quotes(s: &str) -> String {
    let trimmed = s.trim();
    if ((trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
        && trimmed.len() >= 2
    {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── IntentParser::parse ───────────────────────────────────────────────────

    #[test]
    fn parse_empty_intent_returns_empty_spec() {
        let spec = IntentParser::parse("").unwrap();
        assert!(spec.places.is_empty() && spec.actors.is_empty());
    }

    #[test]
    fn parse_place_command() {
        let spec = IntentParser::parse(
            r#"create place zone-1 name "Production Floor" at(0.0, 0.0, 0.0) bounds(50.0, 50.0, 10.0)"#
        ).unwrap();
        assert_eq!(spec.places.len(), 1);
        assert_eq!(spec.places[0].id, "zone-1");
        assert_eq!(spec.places[0].name, "Production Floor"); // quotes stripped
    }

    #[test]
    fn parse_actor_command() {
        let spec = IntentParser::parse(
            r#"create actor hero-1 name Hero role Player in zone-1"#
        ).unwrap();
        assert_eq!(spec.actors.len(), 1);
        assert_eq!(spec.actors[0].id, "hero-1");
        assert_eq!(spec.actors[0].role, "Player");
        assert_eq!(spec.actors[0].place_id, "zone-1");
    }

    #[test]
    fn parse_object_command() {
        let spec = IntentParser::parse(
            r#"create object crate-1 name "Supply Crate" class Box in zone-1"#
        ).unwrap();
        assert_eq!(spec.objects.len(), 1);
        assert_eq!(spec.objects[0].id, "crate-1");
        assert_eq!(spec.objects[0].class, "Box");
    }

    #[test]
    fn parse_relationship_controls_type() {
        let spec = IntentParser::parse(
            "create relationship r1 controls from actor-1 to actor-2"
        ).unwrap();
        assert_eq!(spec.relationships.len(), 1);
        assert!(matches!(spec.relationships[0].rel_type, RelationshipType::Controls));
    }

    #[test]
    fn parse_relationship_custom_type() {
        let spec = IntentParser::parse(
            "create relationship r1 mentors from actor-1 to actor-2"
        ).unwrap();
        assert!(matches!(&spec.relationships[0].rel_type, RelationshipType::Custom(s) if s == "mentors"));
    }

    #[test]
    fn parse_rule_with_severity_error() {
        let spec = IntentParser::parse(
            r#"create rule rule-1 name "Safety Check" expression "machine.temp > 80" severity error"#
        ).unwrap();
        assert_eq!(spec.rules.len(), 1);
        assert_eq!(spec.rules[0].severity, RuleSeverity::Error);
    }

    #[test]
    fn parse_unknown_rule_severity_returns_err() {
        let err = IntentParser::parse(
            r#"create rule r1 name Test expression x severity critical"#
        ).unwrap_err();
        assert!(err.to_string().contains("unknown rule severity"));
    }

    #[test]
    fn parse_unrecognized_command_returns_err() {
        let err = IntentParser::parse("teleport hero to mars").unwrap_err();
        assert!(err.to_string().contains("did not match"));
    }

    #[test]
    fn parse_multiline_with_comments() {
        let intent = "# header comment\n\ncreate actor a1 name Alice role Player in zone-1";
        let spec = IntentParser::parse(intent).unwrap();
        assert_eq!(spec.actors.len(), 1);
    }

    // ── strip_quotes helper ───────────────────────────────────────────────────

    #[test]
    fn strip_quotes_removes_double_quotes() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
    }

    #[test]
    fn strip_quotes_removes_single_quotes() {
        assert_eq!(strip_quotes("'world'"), "world");
    }

    #[test]
    fn strip_quotes_leaves_unquoted_unchanged() {
        assert_eq!(strip_quotes("bare"), "bare");
    }
}
