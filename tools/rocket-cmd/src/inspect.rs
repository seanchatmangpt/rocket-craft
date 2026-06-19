use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::io::{self, Write};
use anyhow::{Result, anyhow};
use serde_json::Value;
use walkdir::WalkDir;
use colored::*;
use dialoguer::{Select, theme::ColorfulTheme};

// Classification categories
const MUD_STATE: &str = "MUD Saved State";
const VERIFICATION_RECEIPT: &str = "E2E Visual Verification Receipt";
const AUDIT_RECEIPT: &str = "GGen Audit / Affidavit Receipt";
const SYNC_RECEIPT: &str = "GGen Sync Receipt";
const COMPLIANCE_RECEIPT: &str = "OCEL Compliance Receipt";
const OCEL_EVENT_LOG: &str = "OCEL Event Log / Event Trail";

pub fn run_inspect(file_path: Option<&str>, summary_only: bool) -> Result<()> {
    let root_dir = get_project_root()?;
    
    if summary_only {
        let classified = scan_workspace(&root_dir);
        print_summary(&classified);
        return Ok(());
    }

    if let Some(path_str) = file_path {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", path_str));
        }
        let rel_path = path.strip_prefix(&root_dir).unwrap_or(path).to_string_lossy().into_owned();
        inspect_file(path, &rel_path)?;
        return Ok(());
    }

    // Interactive REPL Mode
    print_header();
    println!("{}", "Scanning workspace for saved states and receipts...".dimmed());
    let classified = scan_workspace(&root_dir);

    let mut flat_list = Vec::new();
    for (category, files) in &classified {
        for (rel_path, abs_path) in files {
            flat_list.push((category.as_str(), rel_path.clone(), abs_path.clone()));
        }
    }

    if flat_list.is_empty() {
        println!("\n{}", "No developer receipts or save files found in the workspace.".yellow());
        println!("You can generate them by running Playwright tests or saving in the MUD.");
        return Ok(());
    }

    loop {
        println!("\n{}", "Select a file to inspect:".bold());
        println!("{}", "------------------------------------------------------------".dimmed());
        
        let mut selections: Vec<String> = flat_list.iter()
            .map(|(cat, rel_path, _)| format!("{:<32} ➔ {}", cat.green(), rel_path.cyan()))
            .collect();
        selections.push("Quit Inspector".red().to_string());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose a file")
            .default(0)
            .items(&selections)
            .interact_opt()?;

        match selection {
            Some(idx) if idx < flat_list.len() => {
                let (_cat, rel_path, abs_path) = &flat_list[idx];
                if let Err(e) = inspect_file(abs_path, rel_path) {
                    println!("{}", format!("[Error] Failed to inspect file: {}", e).red());
                }
            }
            _ => {
                println!("Goodbye.");
                break;
            }
        }
    }

    Ok(())
}

fn get_project_root() -> Result<PathBuf> {
    // Current directory or relative parent directory
    let current = std::env::current_dir()?;
    if current.join("tools").exists() && current.join("Cargo.toml").exists() {
        Ok(current)
    } else if current.parent().is_some() && current.parent().unwrap().join("tools").exists() {
        Ok(current.parent().unwrap().to_path_buf())
    } else {
        Ok(current)
    }
}

fn print_header() {
    println!("\n{}", "╔══════════════════════════════════════════════════════════╗".bold().cyan());
    println!("{}", "║             ROCKET-CRAFT DEVELOPER INSPECTOR             ║".bold().cyan());
    println!("{}", "║     MUD States  •  Receipts Chain  •  OCEL Event Trails  ║".bold().cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".bold().cyan());
}

fn scan_workspace(root: &Path) -> HashMap<String, Vec<(String, PathBuf)>> {
    let mut classified = HashMap::new();
    classified.insert(MUD_STATE.to_string(), Vec::new());
    classified.insert(VERIFICATION_RECEIPT.to_string(), Vec::new());
    classified.insert(AUDIT_RECEIPT.to_string(), Vec::new());
    classified.insert(SYNC_RECEIPT.to_string(), Vec::new());
    classified.insert(COMPLIANCE_RECEIPT.to_string(), Vec::new());
    classified.insert(OCEL_EVENT_LOG.to_string(), Vec::new());

    let exclude_dirs = ["target", ".git", ".vscode", "node_modules", "blueprint-rs", "unify-rs", "nexus-engine"];

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            e.file_type().is_dir() && !exclude_dirs.contains(&name.as_ref()) || e.file_type().is_file()
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "json") {
            if let Some(category) = classify_file(entry.path()) {
                let rel_path = entry.path().strip_prefix(root).unwrap_or(entry.path()).to_string_lossy().into_owned();
                if let Some(list) = classified.get_mut(&category) {
                    list.push((rel_path, entry.path().to_path_buf()));
                }
            }
        }
    }
    classified
}

fn classify_file(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let data: Value = serde_json::from_str(&content).ok()?;
    
    if !data.is_object() {
        return None;
    }

    let obj = data.as_object()?;

    // Check for MUD Save Data
    if obj.contains_key("player") && obj.contains_key("bloodline") {
        return Some(MUD_STATE.to_string());
    }
    // Check for E2E Verification Receipts
    if obj.contains_key("visualDelta") && (obj.contains_key("prompt") || obj.contains_key("screenshots")) {
        return Some(VERIFICATION_RECEIPT.to_string());
    }
    // Check for GGen Audit/Affidavit Receipts
    if obj.contains_key("events") && obj.contains_key("chain_hash") {
        return Some(AUDIT_RECEIPT.to_string());
    }
    // Check for GGen Sync Receipts
    if obj.contains_key("operation_id") && obj.contains_key("input_hashes") && obj.contains_key("signature") {
        return Some(SYNC_RECEIPT.to_string());
    }
    // Check for OCEL Compliance Receipts
    if obj.contains_key("checkpoint") && obj.contains_key("digest") && obj.contains_key("digest_algorithm") {
        return Some(COMPLIANCE_RECEIPT.to_string());
    }
    // Check for OCEL Event Log
    if obj.contains_key("event_types") && obj.contains_key("events") {
        return Some(OCEL_EVENT_LOG.to_string());
    }

    None
}

fn print_summary(classified: &HashMap<String, Vec<(String, PathBuf)>>) {
    print_header();
    println!("\n{}", "Workspace Receipts and States Summary:".bold());
    println!("{}", "============================================================".dimmed());
    for cat in [MUD_STATE, VERIFICATION_RECEIPT, AUDIT_RECEIPT, SYNC_RECEIPT, COMPLIANCE_RECEIPT, OCEL_EVENT_LOG] {
        let count = classified.get(cat).map_or(0, |l| l.len());
        println!("  {:<35} : {}", cat.bold(), count.to_string().bold());
    }
    println!("{}", "============================================================".dimmed());
}

fn query_json(data: &Value, query_str: &str) -> Option<Value> {
    let parts = query_str.trim().split('.');
    let mut curr = data;
    for p in parts {
        match curr {
            Value::Object(map) => {
                curr = map.get(p)?;
            }
            Value::Array(arr) => {
                if let Ok(idx) = p.parse::<usize>() {
                    curr = arr.get(idx)?;
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(curr.clone())
}

fn format_value(val: &Value) -> String {
    if val.is_object() || val.is_array() {
        serde_json::to_string_pretty(val).unwrap_or_else(|_| val.to_string())
    } else if val.is_string() {
        val.as_str().unwrap().to_string()
    } else {
        val.to_string()
    }
}

fn print_mud_state(rel_path: &str, data: &Value) {
    let player = data.get("player").cloned().unwrap_or(Value::Null);
    let weapon = player.get("weapon").cloned().unwrap_or(Value::Null);
    let shield = player.get("shield").cloned().unwrap_or(Value::Null);
    let perks = player.get("selected_perks").and_then(|v| v.as_array());
    let magic = player.get("magic_unlocks").and_then(|v| v.as_array());
    let loot = player.get("loot_bag").and_then(|v| v.as_array());

    println!("\n{}", format!("Inspection: MUD Saved State ({})", rel_path).bold().green());
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "CHARACTER SHEET:".bold());
    println!("   Name:      {}", player.get("name").and_then(|v| v.as_str()).unwrap_or("N/A").bold().yellow());
    println!("   Level:     {}  |  XP: {}", player.get("level").unwrap_or(&Value::from(1)), player.get("xp").unwrap_or(&Value::from(0)));
    println!("   Bloodline: {} (Rebirths: {})", player.get("bloodline").unwrap_or(&Value::from(0)), data.get("bloodline").unwrap_or(&Value::from(0)));
    println!("   HP:        {} / {}", player.get("health").unwrap_or(&Value::from(0.0)), player.get("max_health").unwrap_or(&Value::from(0.0)));
    println!("   Mana:      {} / {}", player.get("mana").unwrap_or(&Value::from(0.0)), player.get("max_mana").unwrap_or(&Value::from(0.0)));
    println!("   Gold:      {}g", player.get("gold").unwrap_or(&Value::from(0)).to_string().bold().yellow());
    println!("   Combat:    {}", player.get("combat_state").and_then(|v| v.as_str()).unwrap_or("Idle").magenta());
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "CORE STATS:".bold());
    println!("   Attack:  {:<4} |   Defense: {}", player.get("stat_attack").unwrap_or(&Value::from(0)), player.get("stat_defense").unwrap_or(&Value::from(0)));
    println!("   Magic:   {:<4} |   Health:  {}", player.get("stat_magic").unwrap_or(&Value::from(0)), player.get("stat_health").unwrap_or(&Value::from(0)));
    println!("   Unallocated Stat Points: {}", player.get("stat_points").unwrap_or(&Value::from(0)));
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "EQUIPPED GEAR & MAGIC:".bold());
    println!("   Weapon:  {} (ID: {}, Atk Bonus: +{})", 
             weapon.get("name").and_then(|v| v.as_str()).unwrap_or("None").bold().cyan(),
             weapon.get("id").and_then(|v| v.as_str()).unwrap_or("N/A"),
             weapon.get("attack_bonus").unwrap_or(&Value::from(0)));
    println!("   Shield:  {} (Def Bonus: +{})",
             shield.get("name").and_then(|v| v.as_str()).unwrap_or("None").bold().cyan(),
             shield.get("defense_bonus").unwrap_or(&Value::from(0)));
             
    let magic_str = magic.map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("")).collect::<Vec<_>>().join(", ")).unwrap_or_else(|| "None".to_string());
    println!("   Magic:   {}", magic_str);
    
    let perks_str = perks.map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("")).collect::<Vec<_>>().join(", ")).unwrap_or_else(|| "None".to_string());
    println!("   Active Perks: {}", perks_str);
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "INVENTORY LOOT BAG:".bold());
    if let Some(items) = loot {
        if items.is_empty() {
            println!("   (Loot bag empty)");
        } else {
            for (idx, item) in items.iter().enumerate() {
                println!("   [{}] {} (Rarity: {}, Atk: +{})", 
                         idx, 
                         item.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                         item.get("rarity").and_then(|v| v.as_str()).unwrap_or("Common"),
                         item.get("attack_bonus").unwrap_or(&Value::from(0)));
            }
        }
    } else {
        println!("   (Loot bag empty)");
    }
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "ARENA QUEUE:".bold());
    let queue = data.get("arena_queue").and_then(|v| v.as_array());
    let queue_str = queue.map(|arr| arr.iter().map(|v| format!("{}", v.as_str().unwrap_or("").red())).collect::<Vec<_>>().join(" ➔ ")).unwrap_or_else(|| "None".to_string());
    println!("   Next Enemies: {}", queue_str);
}

fn print_verification_receipt(rel_path: &str, data: &Value) {
    let verdict = data.get("verdict").and_then(|v| v.as_str()).unwrap_or("FAIL");
    let verdict_color = if verdict == "PASS" { "PASS".green().bold() } else { "FAIL".red().bold() };
    
    println!("\n{}", format!("Inspection: E2E Visual Verification Receipt ({})", rel_path).bold().green());
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" Timestamp:      {}", data.get("timestamp").and_then(|v| v.as_str()).unwrap_or("N/A"));
    println!(" Verdict:        {}", verdict_color);
    println!(" Visual Delta:   {} pixels", data.get("visualDelta").unwrap_or(&Value::from(0)).to_string().bold().yellow());
    println!(" Package Path:   {}", data.get("packagePath").and_then(|v| v.as_str()).unwrap_or("N/A"));
    println!(" Browser URL:    {}", data.get("browserUrl").and_then(|v| v.as_str()).unwrap_or("N/A"));
    println!(" Signature:      {}", data.get("signature").and_then(|v| v.as_str()).unwrap_or("N/A").dimmed());
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "PROMPT INSTRUCTIONS:".bold());
    let prompt = data.get("prompt").and_then(|v| v.as_str()).unwrap_or("").trim();
    for line in prompt.split('\n') {
        println!("   {}", line.italic().cyan());
    }
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "INPUT ACTUATION TRACE:".bold());
    let trace = data.get("inputTrace").and_then(|v| v.as_array());
    let trace_str = trace.map(|arr| arr.iter().map(|v| format!("{}", v.as_str().unwrap_or("").bold())).collect::<Vec<_>>().join(" ➔ ")).unwrap_or_else(|| "None".to_string());
    println!("   Keys Pressed: {}", trace_str);
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "SCREENSHOTS METADATA:".bold());
    if let Some(obj) = data.get("screenshots").and_then(|v| v.as_object()) {
        for (shot, b64_val) in obj {
            let b64 = b64_val.as_str().unwrap_or("");
            let kb = b64.len() * 3 / 4 / 1024;
            println!("   - {}: {} KB base64 payload", shot.to_uppercase(), kb);
        }
    }
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "CONSOLE LOGS CHECK:".bold());
    let logs = data.get("consoleLogs").and_then(|v| v.as_array());
    if let Some(arr) = logs {
        if arr.is_empty() {
            println!("   No console warnings or errors emitted.");
        } else {
            println!("   Found {} console messages. Press 'l' inside REPL to list them.", arr.len());
        }
    } else {
        println!("   No console warnings or errors emitted.");
    }
}

fn print_audit_receipt(rel_path: &str, data: &Value) {
    println!("\n{}", format!("Inspection: GGen Audit / Affidavit Receipt ({})", rel_path).bold().green());
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" Format Version: {}", data.get("format_version").unwrap_or(&Value::from("N/A")));
    println!(" Chain Hash:     {}", data.get("chain_hash").and_then(|v| v.as_str()).unwrap_or("N/A").bold().cyan());
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "CHRONOLOGICAL EVENT TRAIL:".bold());
    if let Some(events) = data.get("events").and_then(|v| v.as_array()) {
        for ev in events {
            let seq = ev.get("seq").map(|s| s.to_string()).unwrap_or_else(|| "0".to_string());
            let event_type = ev.get("event_type").and_then(|v| v.as_str()).unwrap_or("Unknown");
            println!("  [{}] {}", seq, event_type.yellow());
            println!("      ID:     {}", ev.get("id").and_then(|v| v.as_str()).unwrap_or("N/A"));
            if let Some(objs) = ev.get("objects").and_then(|v| v.as_array()) {
                let obj_strs: Vec<String> = objs.iter().map(|obj| {
                    format!("{} ({}: {})", 
                            obj.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            obj.get("obj_type").and_then(|v| v.as_str()).unwrap_or(""),
                            obj.get("qualifier").and_then(|v| v.as_str()).unwrap_or("").green())
                }).collect();
                println!("      Target: {}", obj_strs.join(", ").bold());
            }
            println!("      Commit: {}", ev.get("payload_commitment").and_then(|v| v.as_str()).unwrap_or("N/A").dimmed());
            println!();
        }
    }
}

fn print_sync_receipt(rel_path: &str, data: &Value) {
    println!("\n{}", format!("Inspection: GGen Sync Receipt ({})", rel_path).bold().green());
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" Operation ID:   {}", data.get("operation_id").unwrap_or(&Value::from("N/A")).to_string().bold());
    println!(" Timestamp:      {}", data.get("timestamp").and_then(|v| v.as_str()).unwrap_or("N/A"));
    println!(" Signature:      {}", data.get("signature").and_then(|v| v.as_str()).unwrap_or("N/A").dimmed());
    println!(" Prev Receipt:   {}", data.get("previous_receipt_hash").and_then(|v| v.as_str()).unwrap_or("N/A"));
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "INPUT HASH COMMITMENTS:".bold());
    if let Some(hashes) = data.get("input_hashes").and_then(|v| v.as_array()) {
        for h in hashes {
            println!("   - {}", h.as_str().unwrap_or("").cyan());
        }
    }
    println!(" {}", "OUTPUT HASH COMMITMENTS:".bold());
    if let Some(hashes) = data.get("output_hashes").and_then(|v| v.as_array()) {
        for h in hashes {
            println!("   - {}", h.as_str().unwrap_or("").cyan());
        }
    } else {
        println!("   - (None)");
    }
}

fn print_compliance_receipt(rel_path: &str, data: &Value) {
    println!("\n{}", format!("Inspection: OCEL Compliance Receipt ({})", rel_path).bold().green());
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" Boundary Path:  {}", data.get("boundary").and_then(|v| v.as_str()).unwrap_or("N/A").bold());
    println!(" Checkpoint ID:  {}", data.get("checkpoint").unwrap_or(&Value::from("N/A")).to_string().yellow());
    println!(" Digest Hash:    {}", data.get("digest").and_then(|v| v.as_str()).unwrap_or("N/A").cyan());
    println!(" Digest Algo:    {}", data.get("digest_algorithm").and_then(|v| v.as_str()).unwrap_or("N/A"));
}

fn print_ocel_log(rel_path: &str, data: &Value) {
    println!("\n{}", format!("Inspection: OCEL Event Log / Event Trail ({})", rel_path).bold().green());
    println!("{}", "------------------------------------------------------------".dimmed());
    if let Some(event_types) = data.get("event_types").and_then(|v| v.as_array()) {
        let et_names: Vec<&str> = event_types.iter().filter_map(|et| et.get("name").and_then(|n| n.as_str())).collect();
        println!(" Event Types:    {}", et_names.join(", "));
    }
    println!("{}", "------------------------------------------------------------".dimmed());
    println!(" {}", "RELATIONSHIP TIMELINE:".bold());
    if let Some(events) = data.get("events").and_then(|v| v.as_array()) {
        for ev in events {
            println!("  [{}] {}", 
                     ev.get("id").and_then(|v| v.as_str()).unwrap_or("").cyan(),
                     ev.get("activity").and_then(|v| v.as_str()).unwrap_or("").bold().yellow());
            if let Some(rels) = ev.get("relationships").and_then(|v| v.as_array()) {
                for r in rels {
                    println!("    └── {}: {}", 
                             r.get("qualifier").and_then(|v| v.as_str()).unwrap_or("").dimmed(),
                             r.get("object_id").and_then(|v| v.as_str()).unwrap_or("").bold());
                }
            }
            println!();
        }
    }
}

fn inspect_file(filepath: &Path, rel_path: &str) -> Result<()> {
    let category = classify_file(filepath).ok_or_else(|| anyhow!("Could not recognize JSON schema for file: {}", rel_path))?;
    let content = fs::read_to_string(filepath)?;
    let data: Value = serde_json::from_str(&content)?;

    match category.as_str() {
        MUD_STATE => print_mud_state(rel_path, &data),
        VERIFICATION_RECEIPT => print_verification_receipt(rel_path, &data),
        AUDIT_RECEIPT => print_audit_receipt(rel_path, &data),
        SYNC_RECEIPT => print_sync_receipt(rel_path, &data),
        COMPLIANCE_RECEIPT => print_compliance_receipt(rel_path, &data),
        OCEL_EVENT_LOG => print_ocel_log(rel_path, &data),
        _ => return Err(anyhow!("Unknown category: {}", category)),
    }

    loop {
        println!("\nOptions: [q] Back to main menu | [raw] Print raw JSON | [query <dot.path>] Query field (e.g. 'query player.gold')");
        if category == VERIFICATION_RECEIPT && data.get("consoleLogs").is_some() {
            println!("         [l] Show console logs");
        }
        
        print!("inspect> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let cmd_parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        let opt = cmd_parts[0].to_lowercase();
        
        if opt == "q" || opt == "quit" || opt == "exit" {
            break;
        } else if opt == "raw" {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        } else if opt == "l" && category == VERIFICATION_RECEIPT && data.get("consoleLogs").is_some() {
            println!("\n{}", "Console Logs Trail:".bold());
            if let Some(arr) = data.get("consoleLogs").and_then(|v| v.as_array()) {
                for entry in arr {
                    println!("  {}", entry.to_string().dimmed());
                }
            }
        } else if opt == "query" {
            if cmd_parts.len() < 2 {
                println!("Usage: query <dot_notation_path> (e.g. query player.name)");
                continue;
            }
            let path = cmd_parts[1];
            if let Some(res) = query_json(&data, path) {
                println!("\n{} = {}", path.bold(), format_value(&res).green());
            } else {
                println!("{}", format!("Field not found or invalid: {}", path).red());
            }
        } else {
            println!("{}", format!("Unknown command: {}", opt).red());
        }
    }

    Ok(())
}
