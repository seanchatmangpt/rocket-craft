use std::path::PathBuf;
use tracing::{info, debug};
use crate::{ConvertedAsset, StagedAsset, PipelineError};

/// Copies converted FBX files into the UE4 Content directory tree.
pub struct Stager {
    /// Root UE4 Content directory (e.g. versions/4.24-Shooter/ShooterGame/Content/Assets/)
    pub content_dir: PathBuf,
}

impl Stager {
    pub fn new(content_dir: PathBuf) -> Self { Self { content_dir } }

    /// Copy the FBX to `content_dir/<asset_name>.fbx`.
    /// Creates content_dir (and parents) if they do not exist.
    /// Returns the full destination path on success.
    pub fn stage(
        &self,
        asset: ConvertedAsset,
    ) -> Result<StagedAsset, (PipelineError, ConvertedAsset)> {
        // 1. Create content_dir if needed
        if let Err(e) = std::fs::create_dir_all(&self.content_dir) {
            return Err((PipelineError::StagingFailed(e), asset));
        }

        let dest = self.content_dir.join(format!("{}.fbx", asset.name()));

        debug!("Staging {} \u{2192} {}", asset.fbx_path.display(), dest.display());

        // 2. Copy the FBX
        if let Err(e) = std::fs::copy(&asset.fbx_path, &dest) {
            return Err((PipelineError::StagingFailed(e), asset));
        }

        info!("Staged: {} ({:.1} KB)", dest.display(),
              std::fs::metadata(&dest).map(|m| m.len() as f64 / 1024.0).unwrap_or(0.0));

        let staged = asset.into_staged(dest);
        Ok(staged)
    }

    /// Stage multiple converted assets, collecting errors rather than aborting.
    pub fn stage_batch(
        &self,
        assets: Vec<ConvertedAsset>,
    ) -> (Vec<StagedAsset>, Vec<(PathBuf, PipelineError)>) {
        let mut staged = Vec::new();
        let mut errors = Vec::new();
        for asset in assets {
            let path = asset.path.clone();
            match self.stage(asset) {
                Ok(s) => staged.push(s),
                Err((e, _)) => errors.push((path, e)),
            }
        }
        (staged, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Format;
    use tempfile::TempDir;

    fn make_converted_asset(tmp: &TempDir) -> ConvertedAsset {
        // Create a fake FBX file
        let fbx_path = tmp.path().join("test_model.fbx");
        std::fs::write(&fbx_path, b"fake fbx content").unwrap();

        ConvertedAsset {
            path: tmp.path().join("test_model.obj"),
            hash: [0u8; 32],
            source_format: Format::Obj,
            fbx_path,
        }
    }

    #[test]
    fn stage_creates_content_dir_if_missing() {
        let tmp = TempDir::new().unwrap();
        let content_dir = tmp.path().join("deep/nested/content");
        let stager = Stager::new(content_dir.clone());

        let asset = make_converted_asset(&tmp);
        let result = stager.stage(asset);

        assert!(result.is_ok());
        assert!(content_dir.exists());
    }

    #[test]
    fn stage_copies_fbx_to_correct_path() {
        let tmp = TempDir::new().unwrap();
        let content_dir = tmp.path().join("content");
        let stager = Stager::new(content_dir.clone());

        let asset = make_converted_asset(&tmp);
        let result = stager.stage(asset).unwrap();

        let expected = content_dir.join("test_model.fbx");
        assert!(expected.exists());
        assert_eq!(result.content_path, expected);
    }

    #[test]
    fn stage_returns_staged_asset_with_correct_content_path() {
        let tmp = TempDir::new().unwrap();
        let content_dir = tmp.path().join("content");
        let stager = Stager::new(content_dir.clone());

        let asset = make_converted_asset(&tmp);
        let staged = stager.stage(asset).unwrap();

        assert_eq!(staged.content_path, content_dir.join("test_model.fbx"));
    }

    #[test]
    fn stage_batch_handles_partial_failures() {
        let tmp = TempDir::new().unwrap();
        let content_dir = tmp.path().join("content");
        let stager = Stager::new(content_dir.clone());

        // Two assets - first has valid fbx, second has missing fbx
        let fbx1 = tmp.path().join("model1.fbx");
        std::fs::write(&fbx1, b"fake fbx").unwrap();

        let asset1 = ConvertedAsset {
            path: tmp.path().join("model1.obj"),
            hash: [0u8; 32],
            source_format: Format::Obj,
            fbx_path: fbx1,
        };

        let asset2 = ConvertedAsset {
            path: tmp.path().join("model2.obj"),
            hash: [1u8; 32],
            source_format: Format::Obj,
            fbx_path: tmp.path().join("nonexistent.fbx"), // missing file
        };

        let (staged, errors) = stager.stage_batch(vec![asset1, asset2]);

        assert_eq!(staged.len(), 1);
        assert_eq!(errors.len(), 1);
    }
}
