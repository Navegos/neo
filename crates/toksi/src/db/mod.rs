// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/db/mod.rs
// created: 2026-11-04
// lastModified: 2026-11-04

pub mod migrate;
pub mod operations;
pub mod schema;

pub use migrate::*;
pub use operations::*;
pub use schema::*;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

pub fn default_db_path() -> PathBuf {
    if let Ok(dir) = std::env::var("TOKSI_DATA_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("toksi.db");
        }
    }
    let data_dir = crate::util::paths::toksi_data_dir();
    PathBuf::from(data_dir).join("toksi.db")
}

pub fn open_default_db() -> Result<ToksiConfigDb, rusqlite::Error> {
    let p = default_db_path();
    let mut db = ToksiConfigDb::open(&p)?;
    let _ = sync_from_legacy_byok_json(&mut db);
    Ok(db)
}

static GLOBAL_DB: OnceLock<Arc<Mutex<ToksiConfigDb>>> = OnceLock::new();

pub fn shared_db() -> Result<Arc<Mutex<ToksiConfigDb>>, rusqlite::Error> {
    if let Some(arc) = GLOBAL_DB.get() {
        return Ok(arc.clone());
    }
    let db = open_default_db()?;
    let arc = Arc::new(Mutex::new(db));
    let _ = GLOBAL_DB.set(arc.clone());
    Ok(arc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_roundtrip() {
        let mut db = ToksiConfigDb::open_in_memory().expect("open memory db");

        let p = ProviderRecord {
            id: "gemini".to_string(),
            name: "Google Gemini".to_string(),
            protocol: "google-gemini".to_string(),
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            is_enabled: true,
            auth_type: "keyring".to_string(),
            metadata_json: "{}".to_string(),
            created_at: 100,
            updated_at: 100,
        };
        db.upsert_provider(&p).expect("upsert provider");

        let retrieved = db.get_provider("gemini").expect("get provider").expect("found");
        assert_eq!(retrieved.name, "Google Gemini");

        let r = ByokRouteRecord {
            id: "claude.default".to_string(),
            token: "tok_12345".to_string(),
            protocol: "anthropic-messages".to_string(),
            upstream: "https://api.anthropic.com".to_string(),
            provider_id: None,
            is_active: true,
            created_at: 100,
            updated_at: 100,
        };
        db.upsert_byok_route(&r).expect("upsert route");

        let route = db.get_byok_route("claude.default").expect("get route").expect("found route");
        assert_eq!(route.upstream, "https://api.anthropic.com");

        let dump_json = db.export_json().expect("export json");
        assert!(dump_json.contains("Google Gemini"));
        assert!(dump_json.contains("claude.default"));

        let mut db2 = ToksiConfigDb::open_in_memory().expect("open memory db 2");
        let imported = db2.import_json(&dump_json, true).expect("import");
        assert_eq!(imported, 2);

        let retrieved2 = db2.get_provider("gemini").expect("get").expect("found");
        assert_eq!(retrieved2.name, "Google Gemini");
    }
}

