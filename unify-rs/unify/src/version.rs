/// Version metadata for a single crate in the unify-rs workspace.
pub struct VersionInfo {
    pub name: &'static str,
    pub version: &'static str,
}

/// Returns version info for all 20 crates in the unify-rs workspace.
pub fn crate_versions() -> Vec<VersionInfo> {
    vec![
        VersionInfo {
            name: "unify",
            version: env!("CARGO_PKG_VERSION"),
        },
        VersionInfo {
            name: "unify-core",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-sem",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-admission",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-receipts",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-macros",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-cli",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-rdf",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-lsp",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-test",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-pm",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-codegen",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-otel",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-ocel",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-wasm",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-ffi",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-config",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-mcp",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-bp",
            version: "0.1.0",
        },
        VersionInfo {
            name: "unify-integration-tests",
            version: "0.1.0",
        },
    ]
}
