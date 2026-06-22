use lsp_types_max::{Diagnostic, DiagnosticSeverity, Position, Range, NumberOrString};

/// GGEN-USD-001: Foreign geometry detected in a part prim.
pub const GGEN_USD_001: &str = "GGEN-USD-001";

/// GGEN-USD-002: Missing `owner_part_id` attribute on a `def Mesh` / `def Xform`.
pub const GGEN_USD_002: &str = "GGEN-USD-002";

/// GGEN-USD-003: Socket prim with a mesh payload.
pub const GGEN_USD_003: &str = "GGEN-USD-003";

const PART_NAMES: &[&str] = &[
    "Head", "Torso", "Arm", "Leg", "Wing", "Blade", "Backpack", "Loadout",
];

fn part_in(prim_name: &str) -> Option<&'static str> {
    PART_NAMES.iter().find(|&&p| prim_name.contains(p)).copied()
}

#[derive(Debug, Clone)]
struct PrimFrame {
    name: String,
    part: Option<&'static str>,
    is_mesh: bool,
    is_xform: bool,
    is_socket: bool,
    has_owner_part_id: bool,
    line: u32,
}

pub struct UsdAnalyzer {
    source: String,
}

impl UsdAnalyzer {
    pub fn new(content: &str) -> Self {
        Self { source: content.to_string() }
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut diags: Vec<Diagnostic> = Vec::new();
        let mut stack: Vec<PrimFrame> = Vec::new();

        let mut create_diag = |line: u32, severity: DiagnosticSeverity, code: &str, message: String| {
            let len = self.source.lines().nth(line as usize).map(|l| l.len()).unwrap_or(0) as u32;
            Diagnostic {
                range: Range {
                    start: Position { line, character: 0 },
                    end: Position { line, character: len },
                },
                severity: Some(severity),
                code: Some(NumberOrString::String(code.to_string())),
                source: Some("ggen-asset-lsp".to_string()),
                message,
                ..Default::default()
            }
        };

        for (idx, raw_line) in self.source.lines().enumerate() {
            let line_no = u32::try_from(idx).unwrap_or(0);
            let trimmed = raw_line.trim();

            if trimmed == "}" {
                if let Some(frame) = stack.pop() {
                    if (frame.is_mesh || frame.is_xform) && !frame.has_owner_part_id {
                        diags.push(create_diag(
                            frame.line,
                            DiagnosticSeverity::ERROR,
                            GGEN_USD_002,
                            format!("GGEN-USD-002: prim \"{}\" is missing a `custom string owner_part_id = \"...\"` attribute.", frame.name),
                        ));
                    }
                }
                continue;
            }

            if trimmed.starts_with("def ") || trimmed == "def" {
                let prim_name = extract_prim_name(trimmed);
                let is_mesh = trimmed.starts_with("def Mesh ");
                let is_xform = trimmed.starts_with("def Xform ");
                let is_socket = prim_name.contains("Socket") || prim_name.contains("socket");
                let part = part_in(&prim_name);

                if let Some(parent_part) = stack.iter().rev().find_map(|f| f.part) {
                    if let Some(child_part) = part {
                        if child_part != parent_part {
                            diags.push(create_diag(
                                line_no,
                                DiagnosticSeverity::ERROR,
                                GGEN_USD_001,
                                format!("GGEN-USD-001: foreign geometry — prim \"{}\" (part: {}) is nested inside a prim belonging to part \"{}\".", prim_name, child_part, parent_part),
                            ));
                        }
                    }
                }

                if is_mesh {
                    if let Some(socket_frame) = stack.iter().rev().find(|f| f.is_socket) {
                        diags.push(create_diag(
                            socket_frame.line,
                            DiagnosticSeverity::WARNING,
                            GGEN_USD_003,
                            format!("GGEN-USD-003: socket prim \"{}\" contains a `def Mesh` payload — sockets must not carry geometry.", socket_frame.name),
                        ));
                    }
                }

                stack.push(PrimFrame {
                    name: prim_name,
                    part,
                    is_mesh,
                    is_xform,
                    is_socket,
                    has_owner_part_id: false,
                    line: line_no,
                });
                continue;
            }

            if trimmed.contains("owner_part_id") {
                if let Some(frame) = stack.last_mut() {
                    frame.has_owner_part_id = true;
                }
            }
        }

        while let Some(frame) = stack.pop() {
            if (frame.is_mesh || frame.is_xform) && !frame.has_owner_part_id {
                diags.push(create_diag(
                    frame.line,
                    DiagnosticSeverity::ERROR,
                    GGEN_USD_002,
                    format!("GGEN-USD-002: prim \"{}\" is missing a `custom string owner_part_id = \"...\"` attribute.", frame.name),
                ));
            }
        }

        diags
    }
}

fn extract_prim_name(trimmed: &str) -> String {
    let mut in_quote = false;
    let mut name = String::new();
    for ch in trimmed.chars() {
        if ch == '"' {
            if in_quote {
                break;
            }
            in_quote = true;
        } else if in_quote {
            name.push(ch);
        }
    }
    name
}
