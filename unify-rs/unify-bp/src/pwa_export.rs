//! BlueprintPwaExporter — generate TypeScript HUD templates, HTML overlays, and
//! JavaScript event bindings from Blueprint event graphs for the pwa-staff frontend.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// BlueprintPwaMetadata
// ---------------------------------------------------------------------------

/// Metadata extracted from a Blueprint that is relevant to PWA UI generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintPwaMetadata {
    /// Name of the source Blueprint.
    pub blueprint_name: String,
    /// All event node names: BeginPlay, EndPlay, Tick, and custom events.
    pub event_names: Vec<String>,
    /// All blueprint variable names.
    pub variable_names: Vec<String>,
    /// Node names that start exec chains (entry points).
    pub exec_entry_points: Vec<String>,
    /// ISO 8601 timestamp of when this metadata was generated.
    pub generated_at: String,
}

// ---------------------------------------------------------------------------
// BlueprintPwaExporter
// ---------------------------------------------------------------------------

/// Generates TypeScript event handler templates, HTML overlays, and JavaScript
/// event bindings for a PWA HUD based on a Blueprint's event nodes and variables.
pub struct BlueprintPwaExporter {
    pub blueprint_name: String,
    pub event_names: Vec<String>,
    pub variable_names: Vec<String>,
}

impl BlueprintPwaExporter {
    /// Create from a [`BlueprintPwaMetadata`] reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use unify_bp::{BlueprintPwaExporter, BlueprintPwaMetadata};
    ///
    /// let meta = BlueprintPwaMetadata {
    ///     blueprint_name: "MyActor".to_string(),
    ///     event_names: vec!["BeginPlay".to_string()],
    ///     variable_names: vec!["Health".to_string()],
    ///     exec_entry_points: vec!["BeginPlay".to_string()],
    ///     generated_at: "2026-06-18T00:00:00Z".to_string(),
    /// };
    /// let exporter = BlueprintPwaExporter::from_metadata(&meta);
    /// assert_eq!(exporter.blueprint_name, "MyActor");
    /// ```
    pub fn from_metadata(meta: &BlueprintPwaMetadata) -> Self {
        Self {
            blueprint_name: meta.blueprint_name.clone(),
            event_names: meta.event_names.clone(),
            variable_names: meta.variable_names.clone(),
        }
    }

    /// Generate a TypeScript class template that mirrors the Blueprint's events as
    /// HUD update methods, matching the `pwa-staff/src/hud.ts` patterns.
    pub fn generate_typescript(&self) -> String {
        let class_name = format!("{}Hud", self.blueprint_name);
        let mut lines: Vec<String> = Vec::new();

        // Header comment
        lines.push(format!(
            "// Auto-generated from Blueprint: {}",
            self.blueprint_name
        ));
        lines.push(format!(
            "// Do not edit — regenerate with: bpgen pwa {}",
            self.blueprint_name
        ));
        lines.push(String::new());
        lines.push(format!("export class {} {{", class_name));

        // Private fields for each variable
        for var in &self.variable_names {
            let field_name = to_camel_case(var);
            let (ts_type, default_val) = infer_ts_type(var);
            lines.push(format!(
                "  private {}: {} = {};",
                field_name, ts_type, default_val
            ));
        }

        // Event handler methods
        for event in &self.event_names {
            lines.push(String::new());
            let method_name = format!("on{}", event);
            if event == "BeginPlay" || event == "begin_play" {
                lines.push(format!("  {}(): void {{", method_name));
                lines.push(format!(
                    "    const overlay = document.getElementById('bp-hud-{}');",
                    self.blueprint_name
                ));
                lines.push("    if (overlay) overlay.style.display = 'block';".to_string());
                lines.push(format!(
                    "    console.log('[{}] {}');",
                    self.blueprint_name, event
                ));
                lines.push("  }".to_string());
            } else {
                lines.push(format!("  {}(amount?: number): void {{", method_name));
                lines.push(format!(
                    "    const eventIndicator = document.getElementById('bp-event-{}');",
                    event
                ));
                lines.push("    if (eventIndicator) {".to_string());
                lines.push("      eventIndicator.classList.add('active');".to_string());
                lines.push(
                    "      setTimeout(() => eventIndicator.classList.remove('active'), 500);"
                        .to_string(),
                );
                lines.push("    }".to_string());
                lines.push(format!(
                    "    console.log('[{}] {}', amount);",
                    self.blueprint_name, event
                ));
                lines.push("  }".to_string());
            }
        }

        // Update methods for each variable
        for var in &self.variable_names {
            let field_name = to_camel_case(var);
            let method_name = format!("update{}", var);
            let (ts_type, _) = infer_ts_type(var);
            lines.push(String::new());
            lines.push(format!("  {}(value: {}): void {{", method_name, ts_type));
            lines.push(format!("    this.{} = value;", field_name));
            // For the first variable that looks like a display value, add DOM update
            if ts_type == "number" {
                lines.push(format!(
                    "    const el = document.getElementById('bp-{}');",
                    var
                ));
                lines.push("    if (el) el.textContent = String(value);".to_string());
            } else if ts_type == "boolean" {
                // booleans get a DOM update too, mirroring the pattern
                lines.push(format!(
                    "    const el = document.getElementById('bp-{}');",
                    var
                ));
                lines.push("    if (el) el.textContent = String(value);".to_string());
            }
            lines.push("  }".to_string());
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Generate an HTML overlay snippet for the Blueprint's variables.
    /// Each variable becomes a `<div id="bp-{name}">` element.
    pub fn generate_html_overlay(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!(
            "<!-- Auto-generated HUD overlay for Blueprint: {} -->",
            self.blueprint_name
        ));
        lines.push(format!(
            r#"<div id="bp-hud-{}" class="bp-hud-overlay">"#,
            self.blueprint_name
        ));

        for var in &self.variable_names {
            let (_, default_val) = infer_ts_type(var);
            // Strip quotes from string defaults for HTML content
            let html_default = default_val.trim_matches('"');
            lines.push(format!(
                r#"  <div class="bp-var" id="bp-{}" data-bp-var="{}">{}</div>"#,
                var, var, html_default
            ));
        }

        lines.push("</div>".to_string());
        lines.join("\n")
    }

    /// Generate a JavaScript event binding script that wires game events to
    /// HUD updates (vanilla JS, no framework dependencies).
    pub fn generate_js_bindings(&self) -> String {
        let class_name = format!("{}Hud", self.blueprint_name);
        let instance_name = to_camel_case(&self.blueprint_name);
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!(
            "// Auto-generated JS event bindings for Blueprint: {}",
            self.blueprint_name
        ));
        lines.push(format!(
            "// Wires game events to HUD instance — vanilla JS, no framework deps"
        ));
        lines.push(String::new());
        lines.push(format!("const {} = new {}();", instance_name, class_name));
        lines.push(String::new());

        for event in &self.event_names {
            let handler = format!("on{}", event);
            let event_key = format!("bp:{}", event);
            lines.push(format!(
                "document.addEventListener('{}', (e) => {{",
                event_key
            ));
            lines.push(format!(
                "  {}.{}(e.detail?.amount);",
                instance_name, handler
            ));
            lines.push("});".to_string());
            lines.push(String::new());
        }

        // Also wire variable updates
        for var in &self.variable_names {
            let update_method = format!("update{}", var);
            let event_key = format!("bp:update:{}", var);
            lines.push(format!(
                "document.addEventListener('{}', (e) => {{",
                event_key
            ));
            lines.push(format!(
                "  {}.{}(e.detail?.value);",
                instance_name, update_method
            ));
            lines.push("});".to_string());
            lines.push(String::new());
        }

        lines.join("\n")
    }

    /// Extract metadata from lists of node names and variable names.
    ///
    /// Event nodes: known lifecycle names (BeginPlay, EndPlay, Tick, and their
    /// snake_case variants) or names prefixed with "On"/"on". Other node names are
    /// not classified as events.
    ///
    /// Exec entry points: canonical lifecycle events that start a top-level execution
    /// chain (BeginPlay / begin_play only).
    pub fn extract_metadata(
        blueprint_name: &str,
        node_names: &[&str],
        variable_names: &[&str],
    ) -> BlueprintPwaMetadata {
        let lifecycle_keywords = [
            "BeginPlay",
            "EndPlay",
            "Tick",
            "begin_play",
            "end_play",
            "tick",
        ];
        let exec_entry_keywords = ["BeginPlay", "begin_play"];

        let mut event_names: Vec<String> = Vec::new();
        let mut exec_entry_points: Vec<String> = Vec::new();

        for &name in node_names {
            let is_known_lifecycle = lifecycle_keywords.iter().any(|&kw| kw == name);
            let is_event_prefix = name.starts_with("On") || name.starts_with("on");
            if is_known_lifecycle || is_event_prefix {
                event_names.push(name.to_string());
            }
            // Only canonical exec-chain entry points (BeginPlay) go into exec_entry_points.
            if exec_entry_keywords.iter().any(|&kw| kw == name) {
                exec_entry_points.push(name.to_string());
            }
        }

        let generated_at = iso8601_now();

        BlueprintPwaMetadata {
            blueprint_name: blueprint_name.to_string(),
            event_names,
            variable_names: variable_names.iter().map(|s| s.to_string()).collect(),
            exec_entry_points,
            generated_at,
        }
    }
}

// ---------------------------------------------------------------------------
// PwaBundle
// ---------------------------------------------------------------------------

/// A complete PWA bundle: TypeScript class + HTML overlay + JS bindings + receipt hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwaBundle {
    /// The generated TypeScript class source.
    pub typescript_class: String,
    /// The generated HTML overlay fragment.
    pub html_overlay: String,
    /// The generated JavaScript event-binding script.
    pub js_bindings: String,
    /// BLAKE3 hash (hex) of typescript_class + html_overlay + js_bindings concatenated.
    pub receipt_hash: String,
    /// Name of the source Blueprint.
    pub blueprint_name: String,
}

impl PwaBundle {
    /// Generate a complete [`PwaBundle`] from a [`BlueprintPwaExporter`].
    pub fn generate(exporter: &BlueprintPwaExporter) -> Self {
        let typescript_class = exporter.generate_typescript();
        let html_overlay = exporter.generate_html_overlay();
        let js_bindings = exporter.generate_js_bindings();

        let combined = format!("{}{}{}", typescript_class, html_overlay, js_bindings);
        let receipt_hash = blake3::hash(combined.as_bytes()).to_hex().to_string();

        Self {
            typescript_class,
            html_overlay,
            js_bindings,
            receipt_hash,
            blueprint_name: exporter.blueprint_name.clone(),
        }
    }

    /// Serialize this bundle to a pretty-printed JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("PwaBundle serialization failed")
    }

    /// Verify that the stored `receipt_hash` matches the current content.
    pub fn verify_receipt(&self) -> bool {
        let combined = format!(
            "{}{}{}",
            self.typescript_class, self.html_overlay, self.js_bindings
        );
        let expected = blake3::hash(combined.as_bytes()).to_hex().to_string();
        self.receipt_hash == expected
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a PascalCase or UPPER_SNAKE name to camelCase for TypeScript fields.
fn to_camel_case(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap().to_lowercase().to_string();
    format!("{}{}", first, chars.as_str())
}

/// Infer a TypeScript type and a sensible default value from a variable name.
/// Returns `(ts_type, default_literal)`.
fn infer_ts_type(name: &str) -> (&'static str, &'static str) {
    let lower = name.to_lowercase();
    if lower.starts_with("is")
        || lower.starts_with("has")
        || lower.starts_with("can")
        || lower.ends_with("alive")
        || lower.ends_with("enabled")
        || lower.ends_with("active")
        || lower.ends_with("bool")
    {
        ("boolean", "true")
    } else if lower.ends_with("name")
        || lower.ends_with("label")
        || lower.ends_with("text")
        || lower.ends_with("tag")
        || lower.ends_with("id")
        || lower.ends_with("key")
    {
        ("string", r#""""#)
    } else {
        // Default: numeric (Health, MaxHealth, Score, Ammo, etc.)
        ("number", "0")
    }
}

/// Return a simple ISO 8601-style timestamp string using system time.
fn iso8601_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Format as YYYY-MM-DDTHH:MM:SSZ (UTC approximation from Unix epoch)
    let (year, month, day, hour, min, sec) = epoch_to_parts(secs);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, min, sec
    )
}

/// Convert Unix epoch seconds to (year, month, day, hour, min, sec) UTC.
fn epoch_to_parts(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sec = secs % 60;
    let mins = secs / 60;
    let min = mins % 60;
    let hours = mins / 60;
    let hour = hours % 24;
    let days = hours / 24;

    // Days since 1970-01-01 → year/month/day using the proleptic Gregorian calendar
    let mut year = 1970u64;
    let mut remaining = days;

    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let months = [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for (i, &days_in_month) in months.iter().enumerate() {
        let dim = if i == 1 && is_leap(year) {
            29
        } else {
            days_in_month
        };
        if remaining < dim {
            break;
        }
        remaining -= dim;
        month += 1;
    }
    let day = remaining + 1;

    (year, month, day, hour, min, sec)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn player_health_meta() -> BlueprintPwaMetadata {
        BlueprintPwaMetadata {
            blueprint_name: "PlayerHealth".to_string(),
            event_names: vec![
                "BeginPlay".to_string(),
                "TakeDamage".to_string(),
                "Death".to_string(),
            ],
            variable_names: vec![
                "Health".to_string(),
                "MaxHealth".to_string(),
                "IsAlive".to_string(),
            ],
            exec_entry_points: vec!["BeginPlay".to_string()],
            generated_at: "2026-06-16T00:00:00Z".to_string(),
        }
    }

    fn player_health_exporter() -> BlueprintPwaExporter {
        BlueprintPwaExporter::from_metadata(&player_health_meta())
    }

    // -----------------------------------------------------------------------
    // from_metadata
    // -----------------------------------------------------------------------

    #[test]
    fn from_metadata_sets_blueprint_name() {
        let exporter = player_health_exporter();
        assert_eq!(exporter.blueprint_name, "PlayerHealth");
    }

    #[test]
    fn from_metadata_copies_event_names() {
        let exporter = player_health_exporter();
        assert_eq!(
            exporter.event_names,
            vec!["BeginPlay", "TakeDamage", "Death"]
        );
    }

    #[test]
    fn from_metadata_copies_variable_names() {
        let exporter = player_health_exporter();
        assert_eq!(
            exporter.variable_names,
            vec!["Health", "MaxHealth", "IsAlive"]
        );
    }

    // -----------------------------------------------------------------------
    // generate_typescript
    // -----------------------------------------------------------------------

    #[test]
    fn generate_typescript_contains_class_name_with_blueprint_name() {
        let ts = player_health_exporter().generate_typescript();
        assert!(
            ts.contains("class PlayerHealthHud"),
            "Expected 'class PlayerHealthHud' in:\n{}",
            ts
        );
    }

    #[test]
    fn generate_typescript_has_method_for_each_event() {
        let ts = player_health_exporter().generate_typescript();
        assert!(ts.contains("onBeginPlay"), "missing onBeginPlay");
        assert!(ts.contains("onTakeDamage"), "missing onTakeDamage");
        assert!(ts.contains("onDeath"), "missing onDeath");
    }

    #[test]
    fn generate_typescript_has_update_method_for_each_variable() {
        let ts = player_health_exporter().generate_typescript();
        assert!(ts.contains("updateHealth"), "missing updateHealth");
        assert!(ts.contains("updateMaxHealth"), "missing updateMaxHealth");
        assert!(ts.contains("updateIsAlive"), "missing updateIsAlive");
    }

    #[test]
    fn generate_typescript_has_balanced_braces() {
        let ts = player_health_exporter().generate_typescript();
        let open = ts.chars().filter(|&c| c == '{').count();
        let close = ts.chars().filter(|&c| c == '}').count();
        assert_eq!(
            open, close,
            "Unbalanced braces: {} open, {} close\n{}",
            open, close, ts
        );
    }

    // -----------------------------------------------------------------------
    // generate_html_overlay
    // -----------------------------------------------------------------------

    #[test]
    fn generate_html_overlay_contains_div_for_each_variable() {
        let html = player_health_exporter().generate_html_overlay();
        assert!(html.contains(r#"id="bp-Health""#), "missing Health div");
        assert!(
            html.contains(r#"id="bp-MaxHealth""#),
            "missing MaxHealth div"
        );
        assert!(html.contains(r#"id="bp-IsAlive""#), "missing IsAlive div");
    }

    #[test]
    fn generate_html_overlay_contains_blueprint_name_in_id() {
        let html = player_health_exporter().generate_html_overlay();
        assert!(
            html.contains("bp-hud-PlayerHealth"),
            "missing bp-hud-PlayerHealth in:\n{}",
            html
        );
    }

    // -----------------------------------------------------------------------
    // generate_js_bindings
    // -----------------------------------------------------------------------

    #[test]
    fn generate_js_bindings_contains_event_binding_for_each_event() {
        let js = player_health_exporter().generate_js_bindings();
        assert!(js.contains("bp:BeginPlay"), "missing bp:BeginPlay binding");
        assert!(
            js.contains("bp:TakeDamage"),
            "missing bp:TakeDamage binding"
        );
        assert!(js.contains("bp:Death"), "missing bp:Death binding");
    }

    #[test]
    fn generate_js_bindings_contains_variable_update_bindings() {
        let js = player_health_exporter().generate_js_bindings();
        assert!(js.contains("bp:update:Health"), "missing bp:update:Health");
        assert!(
            js.contains("bp:update:IsAlive"),
            "missing bp:update:IsAlive"
        );
    }

    // -----------------------------------------------------------------------
    // PwaBundle
    // -----------------------------------------------------------------------

    #[test]
    fn pwa_bundle_generate_produces_non_empty_fields() {
        let bundle = PwaBundle::generate(&player_health_exporter());
        assert!(
            !bundle.typescript_class.is_empty(),
            "typescript_class is empty"
        );
        assert!(!bundle.html_overlay.is_empty(), "html_overlay is empty");
        assert!(!bundle.js_bindings.is_empty(), "js_bindings is empty");
        assert!(!bundle.receipt_hash.is_empty(), "receipt_hash is empty");
    }

    #[test]
    fn pwa_bundle_verify_receipt_returns_true_for_fresh_bundle() {
        let bundle = PwaBundle::generate(&player_health_exporter());
        assert!(
            bundle.verify_receipt(),
            "verify_receipt returned false for fresh bundle"
        );
    }

    #[test]
    fn pwa_bundle_to_json_is_valid_json() {
        let bundle = PwaBundle::generate(&player_health_exporter());
        let json = bundle.to_json();
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("PwaBundle::to_json produced invalid JSON");
        assert_eq!(parsed["blueprint_name"], "PlayerHealth");
    }

    // -----------------------------------------------------------------------
    // extract_metadata
    // -----------------------------------------------------------------------

    #[test]
    fn extract_metadata_classifies_lifecycle_and_on_prefix_as_events() {
        // BeginPlay is a known lifecycle keyword → event.
        // OnTakeDamage starts with "On" → event.
        // Respawn is neither a lifecycle keyword nor On/on-prefixed → not an event.
        let meta = BlueprintPwaExporter::extract_metadata(
            "GameActor",
            &["BeginPlay", "OnTakeDamage", "Respawn"],
            &["Score", "Lives"],
        );
        assert!(
            meta.event_names.contains(&"BeginPlay".to_string()),
            "BeginPlay should be classified as an event"
        );
        assert!(
            meta.event_names.contains(&"OnTakeDamage".to_string()),
            "OnTakeDamage should be classified as an event (On-prefix)"
        );
        assert!(
            !meta.event_names.contains(&"Respawn".to_string()),
            "Respawn should NOT be classified as an event"
        );
    }

    #[test]
    fn extract_metadata_exec_entry_points_contains_only_begin_play() {
        let meta = BlueprintPwaExporter::extract_metadata(
            "GameActor",
            &["BeginPlay", "OnTakeDamage", "Respawn"],
            &["Score"],
        );
        assert!(
            meta.exec_entry_points.contains(&"BeginPlay".to_string()),
            "BeginPlay should be an exec entry point"
        );
        assert!(
            !meta.exec_entry_points.contains(&"OnTakeDamage".to_string()),
            "OnTakeDamage should NOT be an exec entry point"
        );
        assert!(
            !meta.exec_entry_points.contains(&"Respawn".to_string()),
            "Respawn should NOT be an exec entry point"
        );
    }

    #[test]
    fn extract_metadata_returns_correct_variable_names() {
        let meta = BlueprintPwaExporter::extract_metadata(
            "GameActor",
            &["BeginPlay"],
            &["Score", "Lives", "IsActive"],
        );
        assert!(meta.variable_names.contains(&"Score".to_string()));
        assert!(meta.variable_names.contains(&"Lives".to_string()));
        assert!(meta.variable_names.contains(&"IsActive".to_string()));
    }

    #[test]
    fn extract_metadata_sets_blueprint_name() {
        let meta = BlueprintPwaExporter::extract_metadata("MyBlueprint", &[], &[]);
        assert_eq!(meta.blueprint_name, "MyBlueprint");
    }

    #[test]
    fn pwa_bundle_verify_receipt_fails_after_tampering() {
        let mut bundle = PwaBundle::generate(&player_health_exporter());
        bundle.typescript_class.push_str("// tampered");
        assert!(
            !bundle.verify_receipt(),
            "verify_receipt should return false after tampering"
        );
    }
}
