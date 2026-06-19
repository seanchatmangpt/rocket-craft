use crate::errors::GenieError;
use crate::parser::IntentParser;
use crate::receipt_chain::ReceiptChainManager;
use crate::spec::{HistoryEvent, Vector3, WorldSpec};
use regex::Regex;

/// Handles incremental evolution of a manufactured world specification.
pub struct WorldEvolver;

impl WorldEvolver {
    /// Evolve an existing WorldSpec with a new modification intent, preserving existing structures
    pub fn evolve(spec: &WorldSpec, modification_intent: &str) -> Result<WorldSpec, GenieError> {
        let mut new_spec = spec.clone();

        let delete_re = Regex::new(r"^delete\s+(\S+)\s+(\S+)$")
            .map_err(|e| GenieError::Evolution(format!("Regex error: {}", e)))?;
        let delete_generic_re = Regex::new(r"^delete\s+(\S+)$")
            .map_err(|e| GenieError::Evolution(format!("Regex error: {}", e)))?;

        let update_actor_pos_re = Regex::new(
            r"^update\s+actor\s+(\S+)\s+position\s*\(\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*\)$"
        ).map_err(|e| GenieError::Evolution(format!("Regex error: {}", e)))?;
        let update_actor_rot_re = Regex::new(
            r"^update\s+actor\s+(\S+)\s+rotation\s*\(\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*\)$"
        ).map_err(|e| GenieError::Evolution(format!("Regex error: {}", e)))?;

        let update_object_pos_re = Regex::new(
            r"^update\s+object\s+(\S+)\s+position\s*\(\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*\)$"
        ).map_err(|e| GenieError::Evolution(format!("Regex error: {}", e)))?;
        let update_object_rot_re = Regex::new(
            r"^update\s+object\s+(\S+)\s+rotation\s*\(\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*\)$"
        ).map_err(|e| GenieError::Evolution(format!("Regex error: {}", e)))?;

        for (idx, line) in modification_intent.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with("create ") {
                // Parse the single command line as a mini spec
                let temp_spec = IntentParser::parse(trimmed)?;
                // Merge places
                for p in temp_spec.places {
                    if let Some(pos) = new_spec.places.iter().position(|x| x.id == p.id) {
                        new_spec.places[pos] = p;
                    } else {
                        new_spec.places.push(p);
                    }
                }
                // Merge actors
                for a in temp_spec.actors {
                    if let Some(pos) = new_spec.actors.iter().position(|x| x.id == a.id) {
                        new_spec.actors[pos] = a;
                    } else {
                        new_spec.actors.push(a);
                    }
                }
                // Merge objects
                for o in temp_spec.objects {
                    if let Some(pos) = new_spec.objects.iter().position(|x| x.id == o.id) {
                        new_spec.objects[pos] = o;
                    } else {
                        new_spec.objects.push(o);
                    }
                }
                // Merge relationships
                for r in temp_spec.relationships {
                    if let Some(pos) = new_spec.relationships.iter().position(|x| x.id == r.id) {
                        new_spec.relationships[pos] = r;
                    } else {
                        new_spec.relationships.push(r);
                    }
                }
                // Merge rules
                for ru in temp_spec.rules {
                    if let Some(pos) = new_spec.rules.iter().position(|x| x.id == ru.id) {
                        new_spec.rules[pos] = ru;
                    } else {
                        new_spec.rules.push(ru);
                    }
                }
            } else if delete_re.is_match(trimmed) {
                let caps = delete_re.captures(trimmed).unwrap();
                let entity_type = caps[1].to_lowercase();
                let id = &caps[2];
                match entity_type.as_str() {
                    "place" => new_spec.places.retain(|x| x.id != id),
                    "actor" => new_spec.actors.retain(|x| x.id != id),
                    "object" => new_spec.objects.retain(|x| x.id != id),
                    "relationship" => new_spec.relationships.retain(|x| x.id != id),
                    "rule" => new_spec.rules.retain(|x| x.id != id),
                    other => {
                        return Err(GenieError::Evolution(format!(
                            "Line {}: unknown entity type for delete: {}",
                            line_num, other
                        )))
                    }
                }
            } else if delete_generic_re.is_match(trimmed) {
                let caps = delete_generic_re.captures(trimmed).unwrap();
                let id = &caps[1];
                // Remove from all potential collections
                new_spec.places.retain(|x| x.id != id);
                new_spec.actors.retain(|x| x.id != id);
                new_spec.objects.retain(|x| x.id != id);
                new_spec.relationships.retain(|x| x.id != id);
                new_spec.rules.retain(|x| x.id != id);
            } else if update_actor_pos_re.is_match(trimmed) {
                let caps = update_actor_pos_re.captures(trimmed).unwrap();
                let id = &caps[1];
                let x = caps[2].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid x: {}", line_num, e))
                })?;
                let y = caps[3].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid y: {}", line_num, e))
                })?;
                let z = caps[4].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid z: {}", line_num, e))
                })?;
                if let Some(actor) = new_spec.actors.iter_mut().find(|a| a.id == id) {
                    actor.placement.position = Vector3::new(x, y, z);
                } else {
                    return Err(GenieError::Evolution(format!(
                        "Line {}: Actor not found: {}",
                        line_num, id
                    )));
                }
            } else if update_actor_rot_re.is_match(trimmed) {
                let caps = update_actor_rot_re.captures(trimmed).unwrap();
                let id = &caps[1];
                let x = caps[2].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid pitch: {}", line_num, e))
                })?;
                let y = caps[3].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid yaw: {}", line_num, e))
                })?;
                let z = caps[4].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid roll: {}", line_num, e))
                })?;
                if let Some(actor) = new_spec.actors.iter_mut().find(|a| a.id == id) {
                    actor.placement.rotation = Vector3::new(x, y, z);
                } else {
                    return Err(GenieError::Evolution(format!(
                        "Line {}: Actor not found: {}",
                        line_num, id
                    )));
                }
            } else if update_object_pos_re.is_match(trimmed) {
                let caps = update_object_pos_re.captures(trimmed).unwrap();
                let id = &caps[1];
                let x = caps[2].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid x: {}", line_num, e))
                })?;
                let y = caps[3].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid y: {}", line_num, e))
                })?;
                let z = caps[4].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid z: {}", line_num, e))
                })?;
                if let Some(obj) = new_spec.objects.iter_mut().find(|o| o.id == id) {
                    obj.placement.position = Vector3::new(x, y, z);
                } else {
                    return Err(GenieError::Evolution(format!(
                        "Line {}: Object not found: {}",
                        line_num, id
                    )));
                }
            } else if update_object_rot_re.is_match(trimmed) {
                let caps = update_object_rot_re.captures(trimmed).unwrap();
                let id = &caps[1];
                let x = caps[2].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid pitch: {}", line_num, e))
                })?;
                let y = caps[3].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid yaw: {}", line_num, e))
                })?;
                let z = caps[4].parse::<f32>().map_err(|e| {
                    GenieError::Evolution(format!("Line {}: invalid roll: {}", line_num, e))
                })?;
                if let Some(obj) = new_spec.objects.iter_mut().find(|o| o.id == id) {
                    obj.placement.rotation = Vector3::new(x, y, z);
                } else {
                    return Err(GenieError::Evolution(format!(
                        "Line {}: Object not found: {}",
                        line_num, id
                    )));
                }
            } else {
                return Err(GenieError::Evolution(format!(
                    "Line {}: Command not recognized in evolution: '{}'",
                    line_num, trimmed
                )));
            }
        }

        // Add history event for the evolution step
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let event_id = format!("evt_evolve_{}", timestamp_ms);
        let mut new_event = HistoryEvent::new(event_id, timestamp_ms, "Evolve");
        new_event.details.insert(
            "modification_intent".to_string(),
            serde_json::Value::String(modification_intent.to_string()),
        );
        new_spec.history.push(new_event);

        // Regenerate receipt chain with default salt since signature doesn't provide one
        ReceiptChainManager::generate_receipt_chain(&mut new_spec, b"genie_salt")?;

        Ok(new_spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{Actor, Bounds3D, Object, Place, Vector3, WorldSpec};

    fn make_place(id: &str) -> Place {
        Place::new(id, id, Bounds3D::new(Vector3::default(), Vector3::new(5.0, 5.0, 5.0)))
    }

    fn base_spec() -> WorldSpec {
        let mut spec = WorldSpec::new();
        spec.places.push(make_place("room-1"));
        spec.actors.push(Actor::new("hero", "Hero", "Player", "room-1"));
        spec.objects.push(Object::new("box-1", "Crate", "Box", "room-1"));
        spec
    }

    // ── delete commands ───────────────────────────────────────────────────────

    #[test]
    fn delete_actor_by_type_and_id_removes_it() {
        let spec = base_spec();
        let result = WorldEvolver::evolve(&spec, "delete actor hero").unwrap();
        assert!(result.actors.is_empty());
        assert_eq!(result.places.len(), 1); // places untouched
    }

    #[test]
    fn delete_generic_removes_from_any_collection() {
        let spec = base_spec();
        let result = WorldEvolver::evolve(&spec, "delete box-1").unwrap();
        assert!(result.objects.is_empty());
    }

    #[test]
    fn delete_unknown_entity_type_returns_err() {
        let spec = base_spec();
        let err = WorldEvolver::evolve(&spec, "delete widget xyz").unwrap_err();
        assert!(err.to_string().contains("unknown entity type"));
    }

    // ── update position / rotation ────────────────────────────────────────────

    #[test]
    fn update_actor_position_sets_new_coords() {
        let spec = base_spec();
        let result = WorldEvolver::evolve(&spec, "update actor hero position(1.0, 2.0, 3.0)").unwrap();
        let hero = result.actors.iter().find(|a| a.id == "hero").unwrap();
        assert!((hero.placement.position.x - 1.0).abs() < 1e-5);
        assert!((hero.placement.position.y - 2.0).abs() < 1e-5);
        assert!((hero.placement.position.z - 3.0).abs() < 1e-5);
    }

    #[test]
    fn update_actor_position_for_unknown_actor_returns_err() {
        let spec = base_spec();
        let err = WorldEvolver::evolve(&spec, "update actor nobody position(0.0, 0.0, 0.0)").unwrap_err();
        assert!(err.to_string().contains("Actor not found"));
    }

    #[test]
    fn update_object_position_sets_new_coords() {
        let spec = base_spec();
        let result = WorldEvolver::evolve(&spec, "update object box-1 position(7.0, 8.0, 9.0)").unwrap();
        let obj = result.objects.iter().find(|o| o.id == "box-1").unwrap();
        assert!((obj.placement.position.z - 9.0).abs() < 1e-5);
    }

    #[test]
    fn update_object_for_unknown_id_returns_err() {
        let spec = base_spec();
        let err = WorldEvolver::evolve(&spec, "update object ghost position(0.0, 0.0, 0.0)").unwrap_err();
        assert!(err.to_string().contains("Object not found"));
    }

    // ── blank lines / comments / multi-line ──────────────────────────────────

    #[test]
    fn blank_lines_and_hash_comments_are_skipped() {
        let spec = base_spec();
        let intent = "# comment\n\ndelete actor hero";
        let result = WorldEvolver::evolve(&spec, intent).unwrap();
        assert!(result.actors.is_empty());
    }

    #[test]
    fn unrecognized_command_returns_err() {
        let spec = base_spec();
        let err = WorldEvolver::evolve(&spec, "teleport hero to mars").unwrap_err();
        assert!(err.to_string().contains("not recognized"));
    }

    // ── history events added after each evolution ─────────────────────────────

    #[test]
    fn evolve_appends_history_event() {
        let spec = base_spec();
        let before = spec.history.len();
        let result = WorldEvolver::evolve(&spec, "delete actor hero").unwrap();
        assert!(result.history.len() > before);
    }
}
