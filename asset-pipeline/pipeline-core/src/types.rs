use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// ── Typestate asset structs ───────────────────────────────────────────────────
//
// Each struct represents one stage of the pipeline.  Transitions are encoded
// as consuming methods so the type system prevents skipping a stage.
//
// `Converted` and `Staged` carry stage-specific data (fbx_path, content_path),
// which is why we use concrete structs rather than a single generic + PhantomData.

/// An asset that has been found on disk but not yet validated.
#[derive(Debug, Clone)]
pub struct DiscoveredAsset {
    pub path: PathBuf,
    pub hash: [u8; 32],
    pub source_format: Format,
    pub file_size_bytes: u64,
}

/// An asset that has passed format & size validation.
#[derive(Debug, Clone)]
pub struct ValidatedAsset {
    pub path: PathBuf,
    pub hash: [u8; 32],
    pub source_format: Format,
    pub file_size_bytes: u64,
}

/// An asset whose source file has been successfully converted to FBX by Blender.
#[derive(Debug, Clone)]
pub struct ConvertedAsset {
    pub path: PathBuf,
    pub hash: [u8; 32],
    pub source_format: Format,
    /// Absolute path to the generated `.fbx` file.
    pub fbx_path: PathBuf,
}

/// An asset whose FBX has been copied into the Unreal Engine content directory.
#[derive(Debug, Clone)]
pub struct StagedAsset {
    pub path: PathBuf,
    pub hash: [u8; 32],
    /// Absolute path inside the UE4 `Content/` tree.
    pub content_path: PathBuf,
}

// ── State-transition methods ──────────────────────────────────────────────────

impl DiscoveredAsset {
    pub fn new(
        path: PathBuf,
        hash: [u8; 32],
        source_format: Format,
        file_size_bytes: u64,
    ) -> Self {
        Self { path, hash, source_format, file_size_bytes }
    }

    /// Advance to the `Validated` state (caller has already performed checks).
    pub fn into_validated(self) -> ValidatedAsset {
        ValidatedAsset {
            path: self.path,
            hash: self.hash,
            source_format: self.source_format,
            file_size_bytes: self.file_size_bytes,
        }
    }

    /// Returns the bare file stem (e.g. `"my_model"` for `"/foo/my_model.obj"`).
    pub fn name(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }
}

impl ValidatedAsset {
    /// Advance to the `Converted` state once Blender has produced an FBX file.
    pub fn into_converted(self, fbx_path: PathBuf) -> ConvertedAsset {
        ConvertedAsset {
            path: self.path,
            hash: self.hash,
            source_format: self.source_format,
            fbx_path,
        }
    }

    /// Returns the bare file stem.
    pub fn name(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }
}

impl ConvertedAsset {
    /// Advance to the `Staged` state once the FBX has been placed in UE4 Content.
    pub fn into_staged(self, content_path: PathBuf) -> StagedAsset {
        StagedAsset {
            path: self.path,
            hash: self.hash,
            content_path,
        }
    }

    /// Returns the bare file stem.
    pub fn name(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }
}

impl StagedAsset {
    /// Returns the bare file stem.
    pub fn name(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }
}

// ── Format ────────────────────────────────────────────────────────────────────

/// 3D model source formats that the pipeline can ingest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Obj,
    Fbx,
    Stl,
    Dae,
    Gltf,
    Glb,
}

impl Format {
    /// Detect format from a lowercase file extension.  Returns `None` for
    /// extensions that are not supported by this pipeline.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "obj"  => Some(Self::Obj),
            "fbx"  => Some(Self::Fbx),
            "stl"  => Some(Self::Stl),
            "dae"  => Some(Self::Dae),
            "gltf" => Some(Self::Gltf),
            "glb"  => Some(Self::Glb),
            _      => None,
        }
    }

    /// The canonical file extension (without leading dot) for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Obj  => "obj",
            Self::Fbx  => "fbx",
            Self::Stl  => "stl",
            Self::Dae  => "dae",
            Self::Gltf => "gltf",
            Self::Glb  => "glb",
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Obj  => "Wavefront OBJ",
            Self::Fbx  => "Autodesk FBX",
            Self::Stl  => "STL",
            Self::Dae  => "COLLADA DAE",
            Self::Gltf => "glTF",
            Self::Glb  => "glTF Binary (GLB)",
        }
    }

    /// Returns true if this format already is FBX (no Blender conversion needed).
    pub fn is_fbx(&self) -> bool {
        matches!(self, Self::Fbx)
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

// ── PipelineEvent ─────────────────────────────────────────────────────────────

/// Events emitted by the pipeline for broadcast to subscribers and log sinks.
///
/// All variants are `Serialize`-able so they can be written to JSON logs or
/// forwarded over a channel.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum PipelineEvent {
    FileDiscovered {
        path: PathBuf,
    },
    ValidationPassed {
        path: PathBuf,
        /// Lowercase hex string of the BLAKE3 content hash.
        hash_hex: String,
    },
    ValidationFailed {
        path: PathBuf,
        reason: String,
    },
    ConversionStarted {
        path: PathBuf,
    },
    ConversionSucceeded {
        path: PathBuf,
        fbx_path: PathBuf,
    },
    ConversionFailed {
        path: PathBuf,
        reason: String,
    },
    Staged {
        fbx_path: PathBuf,
        content_path: PathBuf,
    },
    Skipped {
        path: PathBuf,
        reason: String,
    },
}

impl PipelineEvent {
    /// The primary path this event concerns, useful for uniform logging.
    pub fn path(&self) -> &PathBuf {
        match self {
            Self::FileDiscovered { path }
            | Self::ValidationPassed { path, .. }
            | Self::ValidationFailed { path, .. }
            | Self::ConversionStarted { path }
            | Self::ConversionSucceeded { path, .. }
            | Self::ConversionFailed { path, .. }
            | Self::Skipped { path, .. } => path,
            Self::Staged { fbx_path, .. } => fbx_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_round_trip() {
        for (ext, expected) in [
            ("obj",  Format::Obj),
            ("OBJ",  Format::Obj),
            ("fbx",  Format::Fbx),
            ("stl",  Format::Stl),
            ("dae",  Format::Dae),
            ("gltf", Format::Gltf),
            ("glb",  Format::Glb),
        ] {
            let got = Format::from_extension(ext).expect("should parse");
            assert_eq!(got, expected, "extension {ext:?}");
            // Check extension() gives back something parseable
            assert!(Format::from_extension(got.extension()).is_some());
        }
    }

    #[test]
    fn format_unsupported_returns_none() {
        assert!(Format::from_extension("png").is_none());
        assert!(Format::from_extension("blend").is_none());
        assert!(Format::from_extension("").is_none());
    }

    #[test]
    fn asset_state_transitions() {
        let discovered = DiscoveredAsset::new(
            PathBuf::from("/tmp/cube.obj"),
            [0u8; 32],
            Format::Obj,
            1024,
        );
        assert_eq!(discovered.name(), "cube");

        let validated = discovered.into_validated();
        assert_eq!(validated.name(), "cube");

        let converted = validated.into_converted(PathBuf::from("/tmp/cube.fbx"));
        assert_eq!(converted.fbx_path, PathBuf::from("/tmp/cube.fbx"));

        let staged = converted.into_staged(PathBuf::from("/content/cube.fbx"));
        assert_eq!(staged.content_path, PathBuf::from("/content/cube.fbx"));
    }

    #[test]
    fn event_serializes_to_json() {
        let event = PipelineEvent::ValidationPassed {
            path: PathBuf::from("/tmp/mesh.obj"),
            hash_hex: "abc123".to_string(),
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["event"], "validation_passed");
        assert_eq!(parsed["hash_hex"], "abc123");
    }
}
