// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/db/migrate.rs
// created: 2026-11-04
// lastModified: 2026-11-04

use std::path::Path;
use crate::util::byok::{byok_routes_path, parse_byok_routes, BYOKRoute};
use crate::util::paths::{read_file_safe, write_file_atomic};
use super::operations::ToksiConfigDb;
use super::schema::ByokRouteRecord;

pub fn sync_from_legacy_byok_json(db: &mut ToksiConfigDb) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let p = byok_routes_path();
    let Some(raw) = read_file_safe(&p) else {
        return Ok(0);
    };
    let routes = parse_byok_routes(&raw)?;
    let mut count = 0;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    for r in routes {
        let rec = ByokRouteRecord {
            id: r.id,
            token: r.token,
            protocol: r.protocol,
            upstream: r.upstream,
            provider_id: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        };
        db.upsert_byok_route(&rec)?;
        count += 1;
    }
    Ok(count)
}

pub fn sync_to_legacy_byok_json(db: &ToksiConfigDb) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let records = db.list_byok_routes()?;
    let routes: Vec<BYOKRoute> = records
        .into_iter()
        .filter(|r| r.is_active)
        .map(|r| BYOKRoute {
            id: r.id,
            token: r.token,
            protocol: r.protocol,
            upstream: r.upstream,
        })
        .collect();

    let content = crate::util::byok::encode_byok_routes(&routes)?;
    let p = byok_routes_path();
    if let Some(parent) = Path::new(&p).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    write_file_atomic(&p, &content)?;
    Ok(())
}

