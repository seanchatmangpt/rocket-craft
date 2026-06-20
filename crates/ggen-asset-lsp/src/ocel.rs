use std::path::Path;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};

static OCEL_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OcelAttribute {
    pub name: String,
    pub time: String,
    pub value: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OcelRelationship {
    pub object_id: String,
    pub qualifier: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OcelEvent {
    pub id: String,
    pub r#type: String,
    pub time: String,
    pub attributes: Vec<OcelAttribute>,
    pub relationships: Vec<OcelRelationship>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OcelObject {
    pub id: String,
    pub r#type: String,
    pub attributes: Vec<OcelAttribute>,
    pub relationships: Vec<OcelRelationship>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OcelEventType {
    pub name: String,
    pub attributes: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OcelObjectType {
    pub name: String,
    pub attributes: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OcelLog {
    pub event_types: Vec<OcelEventType>,
    pub events: Vec<OcelEvent>,
    pub object_types: Vec<OcelObjectType>,
    pub objects: Vec<OcelObject>,
}

fn init_log() -> OcelLog {
    OcelLog {
        event_types: vec![
            OcelEventType { name: "Validate".to_string(), attributes: vec![] },
            OcelEventType { name: "Repair".to_string(), attributes: vec![] },
        ],
        events: vec![],
        object_types: vec![
            OcelObjectType { name: "File".to_string(), attributes: vec![] },
        ],
        objects: vec![],
    }
}

pub fn log_event(
    asset_root: &Path,
    event_type: &str, // "Validate" or "Repair"
    target_file: &Path,
    custom_attrs: Vec<(String, serde_json::Value)>,
) {
    let _lock = OCEL_MUTEX.lock().unwrap();

    let log_path = asset_root.join("ocel").join("lsp_log.json");
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut log = if log_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&log_path) {
            serde_json::from_str::<OcelLog>(&content).unwrap_or_else(|_| init_log())
        } else {
            init_log()
        }
    } else {
        init_log()
    };

    let now_utc = chrono::Utc::now();
    let time_str = now_utc.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let event_id = format!(
        "ev_{}_{}",
        event_type.to_lowercase(),
        now_utc.timestamp_millis()
    );

    let file_name = target_file
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    let file_obj_id = format!("file_{}", file_name);

    // Ensure the File object exists
    if !log.objects.iter().any(|o| o.id == file_obj_id) {
        log.objects.push(OcelObject {
            id: file_obj_id.clone(),
            r#type: "File".to_string(),
            attributes: vec![OcelAttribute {
                name: "path".to_string(),
                time: time_str.clone(),
                value: serde_json::Value::String(target_file.to_string_lossy().to_string()),
            }],
            relationships: vec![],
        });
    }

    let mut attributes = Vec::new();
    for (name, val) in custom_attrs {
        attributes.push(OcelAttribute {
            name,
            time: time_str.clone(),
            value: val,
        });
    }

    let relationships = vec![OcelRelationship {
        object_id: file_obj_id,
        qualifier: "target_file".to_string(),
    }];

    log.events.push(OcelEvent {
        id: event_id,
        r#type: event_type.to_string(),
        time: time_str,
        attributes,
        relationships,
    });

    if let Ok(serialized) = serde_json::to_string_pretty(&log) {
        let _ = std::fs::write(&log_path, serialized);
    }
}
