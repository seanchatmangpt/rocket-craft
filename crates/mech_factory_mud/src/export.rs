use crate::projection::ProjectionRow;
use std::fs;
use std::path::Path;

pub fn export_ue4(manifest: &[ProjectionRow], out_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(out_dir.join("DataTables"))?;
    fs::create_dir_all(out_dir.join("Headers"))?;

    let csvs = [
        "FactoryStations.csv", "WalkthroughRoute.csv", "PartFamilies.csv",
        "SocketTopology.csv", "SkinLayers.csv", "MotionFamilies.csv",
        "SemanticLOD.csv", "ProjectionCommands.csv"
    ];
    for csv in &csvs {
        fs::write(out_dir.join("DataTables").join(csv), "id,name\n1,Test")?;
    }

    let headers = [
        "MechFactoryMudSteps.h", "MechFactoryMudAuthority.h", "MechFactoryMudProjection.h"
    ];
    for h in &headers {
        fs::write(out_dir.join("Headers").join(h), "#pragma once")?;
    }

    fs::write(out_dir.join("ProjectionManifest.json"), serde_json::to_string_pretty(manifest)?)?;
    fs::write(out_dir.join("ReceiptManifest.json"), "[]")?;

    Ok(())
}
