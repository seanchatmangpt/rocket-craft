use crate::projection::ProjectionRow;
use crate::receipt::{ReceiptEvent, verify_receipt_chain};
use std::fs;
use std::path::Path;

pub fn verify_simulation_data(
    receipts: &[ReceiptEvent],
    projections: &[ProjectionRow],
    _ocel_events: &[String],
    walkthrough_csv_path: Option<&Path>,
    _stations_csv_path: Option<&Path>,
) -> Result<(), String> {
    // 1. Verify receipt chain
    if let Err(e) = verify_receipt_chain(receipts) {
        let err_msg = e.to_string();
        if err_msg.contains("Sequence") {
            return Err("RECEIPT_SEQUENCE_GAP".to_string());
        } else if err_msg.contains("prev_hash") {
            return Err("RECEIPT_PREV_HASH_BROKEN".to_string());
        } else if err_msg.contains("Mutated") {
            return Err("RECEIPT_PAYLOAD_MUTATION".to_string());
        } else {
            return Err("RECEIPT_CHAIN_INVALID".to_string());
        }
    }

    // 2. Verify projections have source receipts
    for proj in projections {
        if proj.source_receipt.is_empty() {
            return Err("PROJECTION_WITHOUT_SOURCE_RECEIPT".to_string());
        }
        let found = receipts.iter().any(|r| r.receipt == proj.source_receipt);
        if !found {
            return Err("PROJECTION_WITHOUT_SOURCE_RECEIPT".to_string());
        }
    }

    // 3. Verify OCEL events and objects
    for r in receipts {
        if r.objects.is_empty() {
            return Err("OCEL_EVENT_WITHOUT_OBJECT".to_string());
        }

        let is_part_event = r.event_type == "GenerateFrame"
            || r.event_type == "GenerateArmorPanels"
            || r.event_type == "GenerateSkinLayers"
            || r.event_type == "GenerateSocketTopology"
            || r.event_type == "GenerateMotionFamily";

        if is_part_event {
            let has_part = r.objects.iter().any(|o| o.starts_with("part:"));
            if !has_part {
                return Err("OCEL_PART_EVENT_WITHOUT_PART_OBJECT".to_string());
            }
        }
    }

    // 4. Verify walkthrough route connectivity if CSV path provided
    if let Some(csv_path) = walkthrough_csv_path {
        if csv_path.exists() {
            let content = fs::read_to_string(csv_path)
                .map_err(|e| format!("Failed to read walkthrough route CSV: {}", e))?;

            // Check connectivity
            let mut transitions = Vec::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with("order") {
                    continue;
                }
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 3 {
                    let order = parts[0].parse::<u32>().unwrap_or(0);
                    let node = parts[1].to_string();
                    let next_node = parts[2].to_string();
                    transitions.push((order, node, next_node));
                }
            }

            // Sort by order
            transitions.sort_by_key(|t| t.0);

            // Verify path from Spawn
            if !transitions.is_empty() {
                if transitions[0].1 != "spawn" {
                    return Err("ROUTE_UNREACHABLE".to_string());
                }
                for i in 0..transitions.len() - 1 {
                    if transitions[i].2 != transitions[i + 1].1 {
                        return Err("ROUTE_UNREACHABLE".to_string());
                    }
                }
            }
        }
    }

    Ok(())
}
