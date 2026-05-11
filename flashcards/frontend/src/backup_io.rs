use js_sys::{Array, Date, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, Url};

use crate::model::{AppBackupSnapshot, PersistedState, APP_BACKUP_VERSION};

const BACKUP_FILE_PREFIX: &str = "flashcards-backup";

fn current_timestamp() -> String {
    Date::new_0().to_iso_string().into()
}

pub fn build_backup_snapshot(
    persisted_state: PersistedState,
    datasets: Vec<crate::model::Dataset>,
) -> AppBackupSnapshot {
    build_backup_snapshot_with_timestamp(current_timestamp(), persisted_state, datasets)
}

pub fn build_backup_snapshot_with_timestamp(
    exported_at: String,
    persisted_state: PersistedState,
    datasets: Vec<crate::model::Dataset>,
) -> AppBackupSnapshot {
    AppBackupSnapshot {
        version: APP_BACKUP_VERSION,
        exported_at,
        persisted_state,
        datasets,
    }
}

pub fn backup_file_name() -> String {
    let timestamp = Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_else(|| "unknown-time".to_string())
        .replace(':', "-");

    format!("{BACKUP_FILE_PREFIX}-{timestamp}.json")
}

pub fn serialize_backup_snapshot(
    snapshot: &AppBackupSnapshot,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec_pretty(snapshot)
}

pub fn parse_backup_snapshot(json: &str) -> Result<AppBackupSnapshot, String> {
    let snapshot: AppBackupSnapshot =
        serde_json::from_str(json).map_err(|_| "Backup file is not valid JSON.".to_string())?;
    validate_backup_snapshot(snapshot)
}

pub fn validate_backup_snapshot(snapshot: AppBackupSnapshot) -> Result<AppBackupSnapshot, String> {
    if snapshot.version != APP_BACKUP_VERSION {
        return Err(format!(
            "Unsupported backup version {}. Expected version {}.",
            snapshot.version, APP_BACKUP_VERSION
        ));
    }

    if snapshot.exported_at.trim().is_empty() {
        return Err("Backup file is missing export metadata.".to_string());
    }

    if !snapshot.persisted_state.current_dataset.is_empty()
        && !snapshot
            .datasets
            .iter()
            .any(|dataset| dataset.name == snapshot.persisted_state.current_dataset)
    {
        return Err("Backup file references a missing active wordset.".to_string());
    }

    Ok(snapshot)
}

pub fn trigger_backup_download(bytes: &[u8], file_name: &str) -> Result<(), JsValue> {
    let array = Uint8Array::from(bytes);
    let blob_parts = Array::new();
    blob_parts.push(&array.buffer());
    let blob = Blob::new_with_u8_array_sequence(&blob_parts)?;
    let url = Url::create_object_url_with_blob(&blob)?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document unavailable"))?;
    let anchor = document.create_element("a")?;
    anchor.set_attribute("href", &url)?;
    anchor.set_attribute("download", file_name)?;

    let anchor: web_sys::HtmlElement = anchor.dyn_into()?;
    anchor.click();
    Url::revoke_object_url(&url)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        build_backup_snapshot_with_timestamp, parse_backup_snapshot, serialize_backup_snapshot,
    };
    use crate::model::{Dataset, Flashcard, PersistedState};

    fn sample_state() -> PersistedState {
        PersistedState {
            flashcards: vec![Flashcard {
                word: "你好".to_string(),
                pinyin: Some("nǐ hǎo".to_string()),
                translation: "hello".to_string(),
                known: false,
            }],
            known_cards: vec![],
            current_index: 0,
            stage: Default::default(),
            direction: Default::default(),
            current_dataset: "Basics".to_string(),
        }
    }

    fn sample_datasets() -> Vec<Dataset> {
        vec![Dataset {
            name: "Basics".to_string(),
            flashcards: sample_state().flashcards.clone(),
            known_cards: vec![],
        }]
    }

    #[test]
    fn backup_snapshot_round_trips() {
        let snapshot = build_backup_snapshot_with_timestamp(
            "2026-05-11T10:00:00.000Z".to_string(),
            sample_state(),
            sample_datasets(),
        );
        let json =
            serialize_backup_snapshot(&snapshot).expect("backup serialization should succeed");
        let decoded =
            parse_backup_snapshot(std::str::from_utf8(&json).expect("backup json should be utf-8"))
                .expect("backup parse should succeed");

        assert_eq!(decoded.version, crate::model::APP_BACKUP_VERSION);
        assert_eq!(decoded.persisted_state.current_dataset, "Basics");
        assert_eq!(decoded.datasets.len(), 1);
    }

    #[test]
    fn rejects_unsupported_version() {
        let json = r#"{
            "version": 999,
            "exported_at": "2026-05-11T10:00:00.000Z",
            "persisted_state": {
                "flashcards": [],
                "known_cards": [],
                "current_index": 0,
                "stage": "First",
                "direction": "Normal",
                "current_dataset": ""
            },
            "datasets": []
        }"#;

        let error = parse_backup_snapshot(json).expect_err("unsupported version should fail");
        assert!(error.contains("Unsupported backup version"));
    }

    #[test]
    fn rejects_malformed_json() {
        let error = parse_backup_snapshot("not-json").expect_err("invalid json should fail");
        assert_eq!(error, "Backup file is not valid JSON.");
    }

    #[test]
    fn rejects_missing_active_dataset_reference() {
        let json = r#"{
            "version": 1,
            "exported_at": "2026-05-11T10:00:00.000Z",
            "persisted_state": {
                "flashcards": [],
                "known_cards": [],
                "current_index": 0,
                "stage": "First",
                "direction": "Normal",
                "current_dataset": "Missing"
            },
            "datasets": [
                {
                    "name": "Basics",
                    "flashcards": [],
                    "known_cards": []
                }
            ]
        }"#;

        let error = parse_backup_snapshot(json).expect_err("missing active dataset should fail");
        assert_eq!(error, "Backup file references a missing active wordset.");
    }
}
