//! Developer-experience layer for the `rocket` CLI: shell completions, man-page
//! generation, rich contextual help, and "did you mean?" suggestions.
//!
//! This module is intentionally **self-contained and dependency-light** (std +
//! serde only). It does NOT depend on `clap_complete`, so the completion scripts
//! are generated from a single declarative [`CommandSpec`] source of truth via
//! string templates. That keeps the spec usable from build scripts, tests, docs
//! generators, and the binary alike.
//!
//! # Quick start
//! ```
//! use rocket_sdk::completions::{rocket_command_spec, generate_completions, Shell};
//! let spec = rocket_command_spec();
//! let bash = generate_completions(Shell::Bash, &spec);
//! assert!(bash.contains("doctor"));
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Data model — the single source of truth for the CLI surface.
// ---------------------------------------------------------------------------

/// A command-line flag (e.g. `--project`, `-p`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlagSpec {
    /// Long form without leading dashes, e.g. `"project"`.
    pub long: String,
    /// Optional short form character, e.g. `Some('p')`.
    pub short: Option<char>,
    /// Human-readable description.
    pub about: String,
    /// Whether the flag takes a value (`--project NAME`) vs. is a boolean flag.
    pub takes_value: bool,
    /// Optional static set of completion values for this flag's argument.
    pub value_hints: Vec<String>,
}

impl FlagSpec {
    /// Boolean switch flag (no value).
    pub fn switch(long: &str, short: Option<char>, about: &str) -> Self {
        Self {
            long: long.to_string(),
            short,
            about: about.to_string(),
            takes_value: false,
            value_hints: Vec::new(),
        }
    }

    /// Flag that takes a value.
    pub fn value(long: &str, short: Option<char>, about: &str) -> Self {
        Self {
            long: long.to_string(),
            short,
            about: about.to_string(),
            takes_value: true,
            value_hints: Vec::new(),
        }
    }

    /// Builder: attach a static set of completion hint values.
    pub fn with_hints(mut self, hints: &[&str]) -> Self {
        self.value_hints = hints.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// A positional argument (e.g. the `<file>` in `rocket logs <file>`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgSpec {
    pub name: String,
    pub about: String,
    pub required: bool,
}

impl ArgSpec {
    pub fn new(name: &str, about: &str, required: bool) -> Self {
        Self {
            name: name.to_string(),
            about: about.to_string(),
            required,
        }
    }
}

/// A subcommand. Subcommands may themselves nest subcommands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubcommandSpec {
    pub name: String,
    pub about: String,
    pub flags: Vec<FlagSpec>,
    pub args: Vec<ArgSpec>,
    pub subcommands: Vec<SubcommandSpec>,
    /// Example invocations shown in help / man output.
    pub examples: Vec<String>,
}

impl SubcommandSpec {
    pub fn new(name: &str, about: &str) -> Self {
        Self {
            name: name.to_string(),
            about: about.to_string(),
            flags: Vec::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
            examples: Vec::new(),
        }
    }

    pub fn flag(mut self, flag: FlagSpec) -> Self {
        self.flags.push(flag);
        self
    }

    pub fn arg(mut self, arg: ArgSpec) -> Self {
        self.args.push(arg);
        self
    }

    pub fn subcommand(mut self, sub: SubcommandSpec) -> Self {
        self.subcommands.push(sub);
        self
    }

    pub fn example(mut self, example: &str) -> Self {
        self.examples.push(example.to_string());
        self
    }
}

/// The top-level command (`rocket`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandSpec {
    pub name: String,
    pub about: String,
    pub version: String,
    pub flags: Vec<FlagSpec>,
    pub subcommands: Vec<SubcommandSpec>,
}

impl CommandSpec {
    /// Names of all top-level subcommands.
    pub fn subcommand_names(&self) -> Vec<String> {
        self.subcommands.iter().map(|s| s.name.clone()).collect()
    }

    /// Find a subcommand by name (top level only).
    pub fn find_sub(&self, name: &str) -> Option<&SubcommandSpec> {
        self.subcommands.iter().find(|s| s.name == name)
    }
}

// ---------------------------------------------------------------------------
// The authoritative rocket CLI spec.
// ---------------------------------------------------------------------------

/// Returns the complete, authoritative description of the `rocket` CLI.
///
/// This mirrors `rocket-cmd/src/main.rs` and is the single source of truth used
/// to generate completions, man pages, and help text.
pub fn rocket_command_spec() -> CommandSpec {
    let shell_hints = ["bash", "zsh", "fish", "powershell"];
    let platform_hints = ["Win64", "Linux", "Android", "HTML5", "iOS"];

    CommandSpec {
        name: "rocket".to_string(),
        about: "Rocket Craft Generative Orchestration Tool".to_string(),
        version: "0.1.0".to_string(),
        flags: vec![
            FlagSpec::switch("help", Some('h'), "Print help information"),
            FlagSpec::switch("version", Some('V'), "Print version information"),
        ],
        subcommands: vec![
            SubcommandSpec::new("setup", "Setup the Unreal Engine environment")
                .example("rocket setup"),
            SubcommandSpec::new("sync", "Synchronize project manifest with filesystem")
                .example("rocket sync"),
            SubcommandSpec::new("build", "Build a project target")
                .flag(FlagSpec::value("project", Some('p'), "Project name to build"))
                .flag(FlagSpec::value("target", Some('t'), "Build target"))
                .flag(
                    FlagSpec::value("platform", Some('l'), "Target platform")
                        .with_hints(&platform_hints),
                )
                .example("rocket build -p ShooterGame -t ShooterGame -l Win64")
                .example("rocket build --project SurvivalGame --platform Android"),
            SubcommandSpec::new("audit", "Audit project health and semantic law compliance")
                .example("rocket audit"),
            SubcommandSpec::new("run", "Launch interactive TUI for project management")
                .example("rocket run"),
            SubcommandSpec::new("crypto", "Manage Android keystores and encryption")
                .subcommand(SubcommandSpec::new("generate", "Generate all missing keystores"))
                .subcommand(SubcommandSpec::new("status", "Check status of keystores"))
                .example("rocket crypto generate")
                .example("rocket crypto status"),
            SubcommandSpec::new("clean", "Clean build artifacts (Binaries, Intermediate, Saved)")
                .example("rocket clean"),
            SubcommandSpec::new("pwa", "PWA management and optimization")
                .flag(FlagSpec::value("dir", Some('d'), "Directory containing PWA assets"))
                .flag(FlagSpec::value(
                    "output",
                    Some('o'),
                    "Output minified worker to a different file",
                ))
                .subcommand(SubcommandSpec::new("lint", "Lint and format PWA assets"))
                .subcommand(SubcommandSpec::new("sync", "Generate asset manifest (default)"))
                .example("rocket pwa lint")
                .example("rocket pwa sync --dir pwa-staff"),
            SubcommandSpec::new("info", "Show project information").example("rocket info"),
            SubcommandSpec::new("test", "Run all tests (Rust, Asset validation, etc.)")
                .example("rocket test"),
            SubcommandSpec::new("logs", "Tail Unreal Engine build logs")
                .arg(ArgSpec::new("file", "Specific log file to tail", false))
                .flag(FlagSpec::value(
                    "lines",
                    Some('n'),
                    "Number of initial lines to show",
                ))
                .example("rocket logs")
                .example("rocket logs Build.log --lines 100"),
            SubcommandSpec::new("completions", "Generate shell completions")
                .arg(ArgSpec::new("shell", "The shell to generate completions for", true))
                .example("rocket completions bash")
                .example("rocket completions zsh > ~/.zfunc/_rocket"),
            SubcommandSpec::new("doctor", "Troubleshoot the environment")
                .example("rocket doctor"),
            SubcommandSpec::new(
                "capabilities",
                "List all integrated high-level features (Capabilities)",
            )
            .example("rocket capabilities"),
            SubcommandSpec::new("wasm", "Execute a WASM plugin directly")
                .flag(FlagSpec::value("file", Some('f'), "Path to the WASM file"))
                .example("rocket wasm --file plugins/keystore_law.wasm"),
            // Note: `man` is a DX subcommand surfaced by this module's wiring.
            SubcommandSpec::new("man", "Generate troff man pages for rocket")
                .arg(ArgSpec::new("command", "Subcommand to document (default: all)", false))
                .example("rocket man")
                .example("rocket man build > rocket-build.1"),
        ]
        .into_iter()
        .map(|mut s| {
            // Inject the shell value hints into the `completions` subcommand arg
            // via a flag, keeping the data model uniform for generators.
            if s.name == "completions" {
                s.flags.push(
                    FlagSpec::value("shell", None, "Shell name").with_hints(&shell_hints),
                );
            }
            s
        })
        .collect(),
    }
}

// ---------------------------------------------------------------------------
// Shell enum.
// ---------------------------------------------------------------------------

/// Supported shells for completion generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl Shell {
    /// All supported shells, for iteration in tests / `--help`.
    pub fn all() -> [Shell; 4] {
        [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell]
    }

    /// Lowercase canonical name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "powershell",
        }
    }

    /// Parse a shell from a string (case-insensitive, accepts `pwsh`).
    pub fn parse(s: &str) -> Option<Shell> {
        match s.trim().to_lowercase().as_str() {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            "powershell" | "pwsh" | "ps" => Some(Shell::PowerShell),
            _ => None,
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Completion generation.
// ---------------------------------------------------------------------------

/// Generate a completion script for `shell` from `spec`.
pub fn generate_completions(shell: Shell, spec: &CommandSpec) -> String {
    match shell {
        Shell::Bash => generate_bash(spec),
        Shell::Zsh => generate_zsh(spec),
        Shell::Fish => generate_fish(spec),
        Shell::PowerShell => generate_powershell(spec),
    }
}

fn flag_tokens(flags: &[FlagSpec]) -> String {
    let mut out: Vec<String> = Vec::new();
    for f in flags {
        out.push(format!("--{}", f.long));
        if let Some(c) = f.short {
            out.push(format!("-{}", c));
        }
    }
    out.join(" ")
}

fn generate_bash(spec: &CommandSpec) -> String {
    let bin = &spec.name;
    let subs = spec.subcommand_names().join(" ");
    let mut s = String::new();
    s.push_str(&format!("# bash completion for {bin}\n"));
    s.push_str(&format!("_{bin}() {{\n"));
    s.push_str("    local cur prev words cword\n");
    s.push_str("    COMPREPLY=()\n");
    s.push_str("    cur=\"${COMP_WORDS[COMP_CWORD]}\"\n");
    s.push_str("    prev=\"${COMP_WORDS[COMP_CWORD-1]}\"\n");
    s.push_str(&format!("    local subcommands=\"{subs}\"\n\n"));

    s.push_str("    if [ \"$COMP_CWORD\" -eq 1 ]; then\n");
    s.push_str("        COMPREPLY=( $(compgen -W \"$subcommands\" -- \"$cur\") )\n");
    s.push_str("        return 0\n");
    s.push_str("    fi\n\n");

    s.push_str("    case \"${COMP_WORDS[1]}\" in\n");
    for sub in &spec.subcommands {
        let mut words: Vec<String> = Vec::new();
        for f in &sub.flags {
            words.push(format!("--{}", f.long));
            if let Some(c) = f.short {
                words.push(format!("-{}", c));
            }
        }
        for nested in &sub.subcommands {
            words.push(nested.name.clone());
        }
        s.push_str(&format!("        {})\n", sub.name));
        s.push_str(&format!(
            "            COMPREPLY=( $(compgen -W \"{}\" -- \"$cur\") )\n",
            words.join(" ")
        ));
        s.push_str("            return 0\n");
        s.push_str("            ;;\n");
    }
    s.push_str("    esac\n");
    s.push_str("    return 0\n");
    s.push_str("}\n");
    s.push_str(&format!("complete -F _{bin} {bin}\n"));
    s
}

fn generate_zsh(spec: &CommandSpec) -> String {
    let bin = &spec.name;
    let mut s = String::new();
    s.push_str(&format!("#compdef {bin}\n"));
    s.push_str(&format!("# zsh completion for {bin}\n\n"));
    s.push_str(&format!("_{bin}() {{\n"));
    s.push_str("    local -a commands\n");
    s.push_str("    commands=(\n");
    for sub in &spec.subcommands {
        let about = sub.about.replace('\'', "");
        s.push_str(&format!("        '{}:{}'\n", sub.name, about));
    }
    s.push_str("    )\n\n");
    s.push_str("    _arguments -C \\\n");
    s.push_str("        '1: :->cmds' \\\n");
    s.push_str("        '*:: :->args'\n\n");
    s.push_str("    case $state in\n");
    s.push_str("        cmds)\n");
    s.push_str(&format!("            _describe '{bin} command' commands\n"));
    s.push_str("            ;;\n");
    s.push_str("        args)\n");
    s.push_str("            case $line[1] in\n");
    for sub in &spec.subcommands {
        if sub.flags.is_empty() && sub.subcommands.is_empty() {
            continue;
        }
        s.push_str(&format!("                {})\n", sub.name));
        s.push_str("                    _arguments \\\n");
        for f in &sub.flags {
            let about = f.about.replace('\'', "");
            let val = if f.takes_value { ":value:" } else { "" };
            s.push_str(&format!("                        '--{}[{}]{}' \\\n", f.long, about, val));
        }
        for nested in &sub.subcommands {
            let about = nested.about.replace('\'', "");
            s.push_str(&format!("                        '{}:{}' \\\n", nested.name, about));
        }
        s.push_str("                        && return 0\n");
        s.push_str("                    ;;\n");
    }
    s.push_str("            esac\n");
    s.push_str("            ;;\n");
    s.push_str("    esac\n");
    s.push_str("}\n\n");
    s.push_str(&format!("_{bin} \"$@\"\n"));
    s
}

fn generate_fish(spec: &CommandSpec) -> String {
    let bin = &spec.name;
    let mut s = String::new();
    s.push_str(&format!("# fish completion for {bin}\n"));
    // Disable file completion at the root and provide subcommands.
    s.push_str(&format!(
        "complete -c {bin} -f\n"
    ));
    for sub in &spec.subcommands {
        let about = sub.about.replace('\'', "");
        s.push_str(&format!(
            "complete -c {bin} -n \"__fish_use_subcommand\" -a \"{}\" -d '{}'\n",
            sub.name, about
        ));
    }
    s.push('\n');
    for sub in &spec.subcommands {
        for f in &sub.flags {
            let about = f.about.replace('\'', "");
            let short = f
                .short
                .map(|c| format!(" -s {c}"))
                .unwrap_or_default();
            let value = if f.takes_value { " -r" } else { "" };
            s.push_str(&format!(
                "complete -c {bin} -n \"__fish_seen_subcommand_from {}\"{} -l {}{} -d '{}'\n",
                sub.name, short, f.long, value, about
            ));
        }
        for nested in &sub.subcommands {
            let about = nested.about.replace('\'', "");
            s.push_str(&format!(
                "complete -c {bin} -n \"__fish_seen_subcommand_from {}\" -a \"{}\" -d '{}'\n",
                sub.name, nested.name, about
            ));
        }
    }
    s
}

fn generate_powershell(spec: &CommandSpec) -> String {
    let bin = &spec.name;
    let mut s = String::new();
    s.push_str(&format!("# PowerShell completion for {bin}\n"));
    s.push_str(&format!(
        "Register-ArgumentCompleter -Native -CommandName '{bin}' -ScriptBlock {{\n"
    ));
    s.push_str("    param($wordToComplete, $commandAst, $cursorPosition)\n");
    s.push_str("    $commands = @(\n");
    for sub in &spec.subcommands {
        let about = sub.about.replace('\'', "");
        s.push_str(&format!(
            "        [System.Management.Automation.CompletionResult]::new('{}', '{}', 'ParameterValue', '{}')\n",
            sub.name, sub.name, about
        ));
    }
    s.push_str("    )\n");
    s.push_str("    $commands | Where-Object { $_.CompletionText -like \"$wordToComplete*\" }\n");
    s.push_str("}\n");
    let _ = flag_tokens; // available helper, intentionally retained.
    s
}

// ---------------------------------------------------------------------------
// Man page generation (troff/roff).
// ---------------------------------------------------------------------------

/// Generate a troff/roff man page (section 1) for the whole CLI.
pub fn generate_man_page(spec: &CommandSpec) -> String {
    let upper = spec.name.to_uppercase();
    let mut s = String::new();
    s.push_str(&format!(
        ".TH {} 1 \"\" \"{} {}\" \"User Commands\"\n",
        upper, spec.name, spec.version
    ));
    s.push_str(".SH NAME\n");
    s.push_str(&format!("{} \\- {}\n", spec.name, spec.about));
    s.push_str(".SH SYNOPSIS\n");
    s.push_str(&format!(".B {}\n", spec.name));
    s.push_str("[\\fIGLOBAL OPTIONS\\fR] \\fICOMMAND\\fR [\\fIARGS\\fR]\n");
    s.push_str(".SH DESCRIPTION\n");
    s.push_str(&format!("{}\n", spec.about));

    if !spec.flags.is_empty() {
        s.push_str(".SH GLOBAL OPTIONS\n");
        for f in &spec.flags {
            s.push_str(&man_flag(f));
        }
    }

    s.push_str(".SH COMMANDS\n");
    for sub in &spec.subcommands {
        s.push_str(".TP\n");
        s.push_str(&format!(".B {}\n", sub.name));
        s.push_str(&format!("{}\n", sub.about));

        for f in &sub.flags {
            s.push_str(&man_flag(f));
        }
        for arg in &sub.args {
            let req = if arg.required { "required" } else { "optional" };
            s.push_str(".RS\n.TP\n");
            s.push_str(&format!(".I {} ({})\n", arg.name, req));
            s.push_str(&format!("{}\n", arg.about));
            s.push_str(".RE\n");
        }
        for nested in &sub.subcommands {
            s.push_str(".RS\n.TP\n");
            s.push_str(&format!(".B {} {}\n", sub.name, nested.name));
            s.push_str(&format!("{}\n", nested.about));
            s.push_str(".RE\n");
        }
        if !sub.examples.is_empty() {
            s.push_str(".RS\n");
            for ex in &sub.examples {
                s.push_str(&format!(".EX\n{}\n.EE\n", ex));
            }
            s.push_str(".RE\n");
        }
    }

    s.push_str(".SH EXAMPLES\n");
    for sub in &spec.subcommands {
        for ex in &sub.examples {
            s.push_str(&format!(".EX\n{}\n.EE\n", ex));
        }
    }

    s.push_str(".SH AUTHOR\n");
    s.push_str("Rocket Craft developers.\n");
    s
}

fn man_flag(f: &FlagSpec) -> String {
    let short = f
        .short
        .map(|c| format!("\\fB\\-{c}\\fR, "))
        .unwrap_or_default();
    let value = if f.takes_value { " \\fIVALUE\\fR" } else { "" };
    format!(
        ".TP\n{}\\fB\\-\\-{}\\fR{}\n{}\n",
        short, f.long, value, f.about
    )
}

/// Generate a man page for a single subcommand (e.g. `rocket build`).
pub fn generate_subcommand_man_page(spec: &CommandSpec, sub: &SubcommandSpec) -> String {
    let upper = format!("{}-{}", spec.name, sub.name).to_uppercase();
    let mut s = String::new();
    s.push_str(&format!(
        ".TH {} 1 \"\" \"{} {}\" \"User Commands\"\n",
        upper, spec.name, spec.version
    ));
    s.push_str(".SH NAME\n");
    s.push_str(&format!("{}-{} \\- {}\n", spec.name, sub.name, sub.about));
    s.push_str(".SH SYNOPSIS\n");
    s.push_str(&format!(".B {} {}\n", spec.name, sub.name));
    s.push_str("[\\fIOPTIONS\\fR]\n");
    s.push_str(".SH DESCRIPTION\n");
    s.push_str(&format!("{}\n", sub.about));
    if !sub.flags.is_empty() {
        s.push_str(".SH OPTIONS\n");
        for f in &sub.flags {
            s.push_str(&man_flag(f));
        }
    }
    if !sub.examples.is_empty() {
        s.push_str(".SH EXAMPLES\n");
        for ex in &sub.examples {
            s.push_str(&format!(".EX\n{}\n.EE\n", ex));
        }
    }
    s
}

// ---------------------------------------------------------------------------
// Rich help rendering.
// ---------------------------------------------------------------------------

/// ANSI color helpers; all gated behind a `color` bool so output is plain when
/// piped or when `NO_COLOR` is set.
mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const CYAN: &str = "\x1b[36m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const GREEN: &str = "\x1b[32m";

    pub fn wrap(color: bool, code: &str, text: &str) -> String {
        if color {
            format!("{code}{text}{RESET}")
        } else {
            text.to_string()
        }
    }
}

/// "Common workflows" — curated multi-step recipes for new developers.
fn common_workflows() -> &'static [(&'static str, &'static [&'static str])] {
    &[
        (
            "First-time setup",
            &[
                "rocket setup      # validate UE4 + toolchain",
                "rocket doctor     # diagnose any missing dependencies",
                "rocket sync       # build the project manifest",
            ],
        ),
        (
            "Build & ship a game",
            &[
                "rocket sync",
                "rocket build -p ShooterGame -t ShooterGame -l Win64",
                "rocket audit      # semantic law compliance",
            ],
        ),
        (
            "PWA workflow",
            &[
                "rocket pwa sync   # regenerate asset manifest",
                "rocket pwa lint   # prettier + eslint",
            ],
        ),
        (
            "Android release",
            &[
                "rocket crypto generate",
                "rocket build -p SurvivalGame -l Android",
            ],
        ),
    ]
}

/// Render rich, grouped help. If `topic` names a subcommand, render focused help
/// for that subcommand. Otherwise render the top-level overview with grouped
/// commands, common workflows, and examples.
///
/// `color` toggles ANSI escapes (set `false` when piping / `NO_COLOR`).
pub fn render_help(spec: &CommandSpec, topic: Option<&str>, color: bool) -> String {
    if let Some(t) = topic {
        if let Some(sub) = spec.find_sub(t) {
            return render_subcommand_help(spec, sub, color);
        }
        // Unknown topic: surface a "did you mean?" hint.
        let mut s = String::new();
        s.push_str(&format!("Unknown command: '{t}'\n"));
        if let Some(suggestion) = did_you_mean(t, &spec.subcommand_names()) {
            s.push_str(&ansi::wrap(
                color,
                ansi::YELLOW,
                &format!("Did you mean '{suggestion}'?\n"),
            ));
        }
        s.push_str("\nRun 'rocket --help' for the full command list.\n");
        return s;
    }

    let mut s = String::new();
    s.push_str(&ansi::wrap(
        color,
        ansi::BOLD,
        &format!("{} {}", spec.name, spec.version),
    ));
    s.push('\n');
    s.push_str(&ansi::wrap(color, ansi::DIM, &spec.about));
    s.push_str("\n\n");

    s.push_str(&ansi::wrap(color, ansi::BOLD, "USAGE"));
    s.push('\n');
    s.push_str(&format!("    {} <COMMAND> [OPTIONS]\n\n", spec.name));

    // Group commands into logical sections for scanability.
    let groups: &[(&str, &[&str])] = &[
        ("Project lifecycle", &["setup", "sync", "build", "clean", "run"]),
        ("Quality & compliance", &["audit", "test", "doctor", "wasm"]),
        ("Assets & web", &["pwa", "crypto"]),
        ("Information", &["info", "capabilities", "logs"]),
        ("Developer experience", &["completions", "man"]),
    ];

    for (group_name, names) in groups {
        s.push_str(&ansi::wrap(color, ansi::CYAN, group_name));
        s.push('\n');
        for name in *names {
            if let Some(sub) = spec.find_sub(name) {
                s.push_str(&format!(
                    "    {:<14} {}\n",
                    ansi::wrap(color, ansi::GREEN, &sub.name),
                    sub.about
                ));
            }
        }
        s.push('\n');
    }

    // Common workflows.
    s.push_str(&ansi::wrap(color, ansi::BOLD, "COMMON WORKFLOWS"));
    s.push('\n');
    for (title, steps) in common_workflows() {
        s.push_str(&format!("  {}\n", ansi::wrap(color, ansi::YELLOW, title)));
        for step in *steps {
            s.push_str(&format!("    $ {step}\n"));
        }
        s.push('\n');
    }

    s.push_str(&ansi::wrap(color, ansi::DIM, "Run 'rocket man' to view full manual pages, or 'rocket completions <shell>' to enable tab completion."));
    s.push('\n');
    s
}

fn render_subcommand_help(spec: &CommandSpec, sub: &SubcommandSpec, color: bool) -> String {
    let mut s = String::new();
    s.push_str(&ansi::wrap(
        color,
        ansi::BOLD,
        &format!("{} {}", spec.name, sub.name),
    ));
    s.push('\n');
    s.push_str(&ansi::wrap(color, ansi::DIM, &sub.about));
    s.push_str("\n\n");

    s.push_str(&ansi::wrap(color, ansi::BOLD, "USAGE"));
    s.push('\n');
    let args_usage: String = sub
        .args
        .iter()
        .map(|a| {
            if a.required {
                format!(" <{}>", a.name)
            } else {
                format!(" [{}]", a.name)
            }
        })
        .collect();
    let opts = if sub.flags.is_empty() { "" } else { " [OPTIONS]" };
    s.push_str(&format!("    {} {}{}{}\n\n", spec.name, sub.name, opts, args_usage));

    if !sub.args.is_empty() {
        s.push_str(&ansi::wrap(color, ansi::BOLD, "ARGUMENTS"));
        s.push('\n');
        for a in &sub.args {
            s.push_str(&format!(
                "    {:<14} {}\n",
                ansi::wrap(color, ansi::GREEN, &a.name),
                a.about
            ));
        }
        s.push('\n');
    }

    if !sub.flags.is_empty() {
        s.push_str(&ansi::wrap(color, ansi::BOLD, "OPTIONS"));
        s.push('\n');
        for f in &sub.flags {
            let short = f.short.map(|c| format!("-{c}, ")).unwrap_or_default();
            let val = if f.takes_value { " <VALUE>" } else { "" };
            let flag = format!("{}--{}{}", short, f.long, val);
            s.push_str(&format!(
                "    {:<22} {}\n",
                ansi::wrap(color, ansi::GREEN, &flag),
                f.about
            ));
        }
        s.push('\n');
    }

    if !sub.subcommands.is_empty() {
        s.push_str(&ansi::wrap(color, ansi::BOLD, "SUBCOMMANDS"));
        s.push('\n');
        for nested in &sub.subcommands {
            s.push_str(&format!(
                "    {:<14} {}\n",
                ansi::wrap(color, ansi::GREEN, &nested.name),
                nested.about
            ));
        }
        s.push('\n');
    }

    if !sub.examples.is_empty() {
        s.push_str(&ansi::wrap(color, ansi::BOLD, "EXAMPLES"));
        s.push('\n');
        for ex in &sub.examples {
            s.push_str(&format!("    $ {ex}\n"));
        }
        s.push('\n');
    }
    s
}

// ---------------------------------------------------------------------------
// "Did you mean?" — Levenshtein-based fuzzy suggestion.
// ---------------------------------------------------------------------------

/// Classic Levenshtein edit distance between two strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }

    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr: Vec<usize> = vec![0; m + 1];

    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Suggest the closest candidate to `input` if one is "close enough".
///
/// Uses Levenshtein distance with a threshold scaled to the input length, so
/// short typos suggest aggressively and long strings require closer matches.
pub fn did_you_mean(input: &str, candidates: &[String]) -> Option<String> {
    let input_lc = input.to_lowercase();
    // Threshold: roughly 1 edit per 3 chars, minimum 1, capped at 3.
    let threshold = input.chars().count().div_ceil(3).clamp(1, 3);

    let mut best: Option<(usize, &String)> = None;
    for cand in candidates {
        let dist = levenshtein(&input_lc, &cand.to_lowercase());
        // A prefix match is a strong signal even if distance is larger.
        let effective = if cand.to_lowercase().starts_with(&input_lc) && !input_lc.is_empty() {
            0
        } else {
            dist
        };
        match best {
            Some((best_dist, _)) if effective >= best_dist => {}
            _ => best = Some((effective, cand)),
        }
    }

    best.and_then(|(dist, cand)| {
        if dist <= threshold {
            Some(cand.clone())
        } else {
            None
        }
    })
}

// ---------------------------------------------------------------------------
// Install instructions.
// ---------------------------------------------------------------------------

/// Tell the user exactly where to install the completion script for `shell`.
pub fn install_instructions(shell: Shell) -> String {
    match shell {
        Shell::Bash => "\
# Bash: add completions to your user completion dir
rocket completions bash > ~/.local/share/bash-completion/completions/rocket
# Or, system-wide:
rocket completions bash | sudo tee /etc/bash_completion.d/rocket > /dev/null
# Then restart your shell or: source ~/.bashrc"
            .to_string(),
        Shell::Zsh => "\
# Zsh: drop the script on your $fpath, then rebuild the completion cache
mkdir -p ~/.zfunc
rocket completions zsh > ~/.zfunc/_rocket
# Ensure ~/.zfunc is on fpath (add to ~/.zshrc BEFORE compinit):
#   fpath=(~/.zfunc $fpath)
#   autoload -Uz compinit && compinit
# Then restart your shell or: exec zsh"
            .to_string(),
        Shell::Fish => "\
# Fish: completions live in the per-user completions directory
rocket completions fish > ~/.config/fish/completions/rocket.fish
# Fish loads them automatically on next prompt."
            .to_string(),
        Shell::PowerShell => "\
# PowerShell: append the completer to your profile
rocket completions powershell | Out-String | Add-Content $PROFILE
# Then reload: . $PROFILE"
            .to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> CommandSpec {
        rocket_command_spec()
    }

    #[test]
    fn spec_covers_all_real_subcommands() {
        let names = spec().subcommand_names();
        for expected in [
            "setup",
            "sync",
            "build",
            "audit",
            "run",
            "crypto",
            "clean",
            "pwa",
            "info",
            "test",
            "logs",
            "completions",
            "doctor",
            "capabilities",
            "wasm",
        ] {
            assert!(names.contains(&expected.to_string()), "missing {expected}");
        }
    }

    #[test]
    fn build_has_project_target_platform_flags() {
        let build = spec().find_sub("build").unwrap().clone();
        let longs: Vec<&str> = build.flags.iter().map(|f| f.long.as_str()).collect();
        assert!(longs.contains(&"project"));
        assert!(longs.contains(&"target"));
        assert!(longs.contains(&"platform"));
    }

    #[test]
    fn crypto_has_generate_and_status_subcommands() {
        let crypto = spec().find_sub("crypto").unwrap().clone();
        let subs: Vec<&str> = crypto.subcommands.iter().map(|s| s.name.as_str()).collect();
        assert!(subs.contains(&"generate"));
        assert!(subs.contains(&"status"));
    }

    #[test]
    fn shell_roundtrip_and_parse() {
        for sh in Shell::all() {
            assert_eq!(Shell::parse(sh.as_str()), Some(sh));
        }
        assert_eq!(Shell::parse("PWSH"), Some(Shell::PowerShell));
        assert_eq!(Shell::parse("nushell"), None);
    }

    #[test]
    fn bash_completion_contains_all_subcommands() {
        let out = generate_completions(Shell::Bash, &spec());
        assert!(out.contains("complete -F _rocket rocket"));
        for name in spec().subcommand_names() {
            assert!(out.contains(&name), "bash missing {name}");
        }
    }

    #[test]
    fn zsh_completion_contains_compdef_and_commands() {
        let out = generate_completions(Shell::Zsh, &spec());
        assert!(out.contains("#compdef rocket"));
        assert!(out.contains("build:"));
        assert!(out.contains("doctor:"));
    }

    #[test]
    fn fish_completion_uses_subcommand_predicate() {
        let out = generate_completions(Shell::Fish, &spec());
        assert!(out.contains("__fish_use_subcommand"));
        assert!(out.contains("-a \"build\""));
        assert!(out.contains("__fish_seen_subcommand_from build"));
    }

    #[test]
    fn powershell_completion_registers_completer() {
        let out = generate_completions(Shell::PowerShell, &spec());
        assert!(out.contains("Register-ArgumentCompleter"));
        assert!(out.contains("'doctor'"));
    }

    #[test]
    fn all_shells_generate_nonempty_scripts() {
        for sh in Shell::all() {
            let out = generate_completions(sh, &spec());
            assert!(!out.is_empty(), "{sh} produced empty output");
            assert!(out.contains("rocket"), "{sh} missing bin name");
        }
    }

    #[test]
    fn man_page_has_sections_and_commands() {
        let out = generate_man_page(&spec());
        assert!(out.contains(".TH ROCKET 1"));
        assert!(out.contains(".SH NAME"));
        assert!(out.contains(".SH COMMANDS"));
        assert!(out.contains(".SH EXAMPLES"));
        assert!(out.contains(".B build"));
        assert!(out.contains(".B doctor"));
    }

    #[test]
    fn subcommand_man_page_renders() {
        let s = spec();
        let build = s.find_sub("build").unwrap();
        let out = generate_subcommand_man_page(&s, build);
        assert!(out.contains(".TH ROCKET-BUILD 1"));
        assert!(out.contains("project"));
        assert!(out.contains(".SH EXAMPLES"));
    }

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("doctr", "doctor"), 1);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn did_you_mean_suggests_close_typos() {
        let cands = spec().subcommand_names();
        assert_eq!(did_you_mean("doctr", &cands), Some("doctor".to_string()));
        assert_eq!(did_you_mean("biuld", &cands), Some("build".to_string()));
        assert_eq!(did_you_mean("setp", &cands), Some("setup".to_string()));
        assert_eq!(did_you_mean("audi", &cands), Some("audit".to_string()));
    }

    #[test]
    fn did_you_mean_returns_none_for_garbage() {
        let cands = spec().subcommand_names();
        assert_eq!(did_you_mean("xyzzyqwerty", &cands), None);
    }

    #[test]
    fn did_you_mean_prefix_match() {
        let cands = spec().subcommand_names();
        // "comp" is a prefix of "completions".
        assert_eq!(did_you_mean("comp", &cands), Some("completions".to_string()));
    }

    #[test]
    fn render_help_overview_has_workflows_and_groups() {
        let out = render_help(&spec(), None, false);
        assert!(out.contains("COMMON WORKFLOWS"));
        assert!(out.contains("Project lifecycle"));
        assert!(out.contains("setup"));
        assert!(out.contains("build"));
        // No ANSI when color=false.
        assert!(!out.contains('\x1b'));
    }

    #[test]
    fn render_help_colorized_emits_ansi() {
        let out = render_help(&spec(), None, true);
        assert!(out.contains('\x1b'));
    }

    #[test]
    fn render_help_topic_renders_subcommand() {
        let out = render_help(&spec(), Some("build"), false);
        assert!(out.contains("USAGE"));
        assert!(out.contains("--project"));
        assert!(out.contains("EXAMPLES"));
    }

    #[test]
    fn render_help_unknown_topic_suggests() {
        let out = render_help(&spec(), Some("biuld"), false);
        assert!(out.contains("Unknown command"));
        assert!(out.contains("build"));
    }

    #[test]
    fn install_instructions_mention_paths() {
        assert!(install_instructions(Shell::Bash).contains("bash-completion"));
        assert!(install_instructions(Shell::Zsh).contains("fpath") || install_instructions(Shell::Zsh).contains("_rocket"));
        assert!(install_instructions(Shell::Fish).contains("completions/rocket.fish"));
        assert!(install_instructions(Shell::PowerShell).contains("$PROFILE"));
    }

    #[test]
    fn spec_serializes_to_json() {
        let json = serde_json::to_string(&spec()).unwrap();
        assert!(json.contains("rocket"));
        let back: CommandSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "rocket");
    }
}
