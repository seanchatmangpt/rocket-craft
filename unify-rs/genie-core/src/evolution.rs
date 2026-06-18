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
