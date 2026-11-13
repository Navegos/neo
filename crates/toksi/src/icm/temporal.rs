// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/temporal.rs
// created: 2026-11-13

#![deny(unsafe_code)]

use chrono::{
    DateTime, Datelike, Days, FixedOffset, Months, NaiveDate, NaiveDateTime, Offset, TimeZone, Utc,
};

use crate::icm::schema::{IcmError, IcmResult, Importance, TimeWindow};

const MIN_YEAR: i32 = 1990;
const MAX_YEAR_BOUND: i32 = 2100;
const MIN_INSTANT_YEAR: i32 = 1;
const MAX_INSTANT_YEAR: i32 = 9999;

/// Parse an instant supplied by a caller (RFC 3339 or YYYY-MM-DD[THH:MM:SS]).
pub fn parse_instant(s: &str) -> IcmResult<DateTime<Utc>> {
    let s = s.trim();
    let parsed = if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        Some(dt.with_timezone(&Utc))
    } else {
        ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S"]
            .iter()
            .find_map(|fmt| NaiveDateTime::parse_from_str(s, fmt).ok())
            .or_else(|| {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .ok()
                    .and_then(|day| day.and_hms_opt(0, 0, 0))
            })
            .map(|naive| Utc.from_utc_datetime(&naive))
    };

    let Some(instant) = parsed else {
        return Err(IcmError::InvalidInput(format!(
            "invalid date '{s}' (expected RFC 3339, YYYY-MM-DDTHH:MM:SS or YYYY-MM-DD)"
        )));
    };

    if !(MIN_INSTANT_YEAR..=MAX_INSTANT_YEAR).contains(&instant.year()) {
        return Err(IcmError::InvalidInput(format!(
            "invalid date '{s}': year {} out of range",
            instant.year()
        )));
    }

    Ok(instant)
}

/// Calculate temporal decay factor on memory weight.
/// Critical importance is immune to decay.
/// Other importance levels decay according to:
/// weight = initial_weight * MAX(0.0, 1.0 - (1.0 - decay_factor) * mult / (1.0 + min(access_count, 5) * 0.1))
pub fn calculate_temporal_decay(
    initial_weight: f32,
    decay_factor: f32,
    importance: Importance,
    access_count: u32,
) -> f32 {
    if importance == Importance::Critical {
        return initial_weight;
    }

    let mult = match importance {
        Importance::High => 0.5,
        Importance::Medium => 1.0,
        Importance::Low => 2.0,
        Importance::Critical => 0.0,
    };

    let capped_access = access_count.min(5) as f32;
    let denom = 1.0 + capped_access * 0.1;
    let factor = (1.0 - (1.0 - decay_factor) * mult / denom).max(0.0);
    initial_weight * factor
}

/// Parse a time window from natural language query ("yesterday", "last week", "in 2023", etc.)
pub fn parse_query_window(query: &str, now: DateTime<Utc>) -> Option<TimeWindow> {
    parse_query_window_at(query, now, Utc.fix())
}

pub fn parse_query_window_at(
    query: &str,
    now: DateTime<Utc>,
    offset: FixedOffset,
) -> Option<TimeWindow> {
    let today = now.with_timezone(&offset).date_naive();
    let q = query.to_lowercase();
    let tokens: Vec<&str> = q.split_whitespace().collect();

    // 1. Direct ISO date YYYY-MM-DD
    for token in &tokens {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        if let Ok(d) = NaiveDate::parse_from_str(clean, "%Y-%m-%d") {
            return day_window(d, offset);
        }
    }

    // 2. Relative day expressions
    if q.contains("today") || q.contains("aujourd'hui") {
        return day_window(today, offset);
    }
    if q.contains("yesterday") || q.contains("hier") {
        let y = today.pred_opt()?;
        return day_window(y, offset);
    }
    if q.contains("avant-hier") || q.contains("day before yesterday") {
        let prev = today.checked_sub_days(Days::new(2))?;
        return day_window(prev, offset);
    }

    // 3. Civil periods: last/this week, month, year
    if q.contains("last week") || q.contains("semaine dernière") {
        let since_monday = u64::from(today.weekday().num_days_from_monday());
        let current_monday = today.checked_sub_days(Days::new(since_monday))?;
        let last_monday = current_monday.checked_sub_days(Days::new(7))?;
        return span_window(last_monday, current_monday, offset);
    }
    if q.contains("this week") || q.contains("cette semaine") {
        let since_monday = u64::from(today.weekday().num_days_from_monday());
        let monday = today.checked_sub_days(Days::new(since_monday))?;
        let next_monday = monday.checked_add_days(Days::new(7))?;
        return span_window(monday, next_monday, offset);
    }
    if q.contains("last month") || q.contains("mois dernier") {
        let first_this_month = today.with_day(1)?;
        let first_last_month = first_this_month.checked_sub_months(Months::new(1))?;
        return span_window(first_last_month, first_this_month, offset);
    }
    if q.contains("this month") || q.contains("ce mois") {
        let first_this_month = today.with_day(1)?;
        let first_next_month = first_this_month.checked_add_months(Months::new(1))?;
        return span_window(first_this_month, first_next_month, offset);
    }
    if q.contains("last year") || q.contains("l'an dernier") || q.contains("année dernière") {
        let y = today.year().checked_sub(1)?;
        let start = NaiveDate::from_ymd_opt(y, 1, 1)?;
        let end = NaiveDate::from_ymd_opt(today.year(), 1, 1)?;
        return span_window(start, end, offset);
    }
    if q.contains("this year") || q.contains("cette année") {
        let start = NaiveDate::from_ymd_opt(today.year(), 1, 1)?;
        let end = NaiveDate::from_ymd_opt(today.year().checked_add(1)?, 1, 1)?;
        return span_window(start, end, offset);
    }

    // 4. "N days/weeks/months/years ago"
    for i in 0..tokens.len() {
        if i + 2 < tokens.len() && tokens[i + 2] == "ago" {
            if let Ok(n) = tokens[i].parse::<u64>() {
                let unit = tokens[i + 1].trim_end_matches('s');
                match unit {
                    "day" => {
                        let center = today.checked_sub_days(Days::new(n))?;
                        let start = center.pred_opt()?;
                        let end = center.succ_opt()?;
                        return span_window(start, end, offset);
                    }
                    "week" => {
                        let center = today.checked_sub_days(Days::new(n * 7))?;
                        let start = center.checked_sub_days(Days::new(4))?;
                        let end = center.checked_add_days(Days::new(4))?;
                        return span_window(start, end, offset);
                    }
                    "month" => {
                        let center = today.checked_sub_months(Months::new(n as u32))?;
                        let start = center.checked_sub_days(Days::new(15))?;
                        let end = center.checked_add_days(Days::new(15))?;
                        return span_window(start, end, offset);
                    }
                    "year" => {
                        let center_year = today.year().checked_sub(n as i32)?;
                        let start = NaiveDate::from_ymd_opt(center_year, 1, 1)?;
                        let end = NaiveDate::from_ymd_opt(center_year.checked_add(1)?, 1, 1)?;
                        return span_window(start, end, offset);
                    }
                    _ => {}
                }
            }
        }
    }

    // 5. Named month ("in March 2024", "January 2023", etc.)
    let months = [
        ("january", 1), ("february", 2), ("march", 3), ("april", 4),
        ("may", 5), ("june", 6), ("july", 7), ("august", 8),
        ("september", 9), ("october", 10), ("november", 11), ("december", 12),
        ("janvier", 1), ("février", 2), ("mars", 3), ("avril", 4),
        ("mai", 5), ("juin", 6), ("juillet", 7), ("août", 8),
        ("septembre", 9), ("octobre", 10), ("novembre", 11), ("décembre", 12),
    ];

    for (m_name, m_num) in &months {
        if q.contains(m_name) {
            // Find year if specified nearby
            let mut matched_year = today.year();
            for token in &tokens {
                if let Ok(y) = token.parse::<i32>() {
                    if (MIN_YEAR..=MAX_YEAR_BOUND).contains(&y) {
                        matched_year = y;
                        break;
                    }
                }
            }
            if let Some(start) = NaiveDate::from_ymd_opt(matched_year, *m_num, 1) {
                if let Some(end) = start.checked_add_months(Months::new(1)) {
                    return span_window(start, end, offset);
                }
            }
        }
    }

    // 6. "in YYYY" / "en YYYY"
    for i in 0..tokens.len() {
        if (tokens[i] == "in" || tokens[i] == "en") && i + 1 < tokens.len() {
            let clean = tokens[i + 1].trim_matches(|c: char| !c.is_numeric());
            if let Ok(y) = clean.parse::<i32>() {
                if (MIN_YEAR..=MAX_YEAR_BOUND).contains(&y) && y <= today.year() {
                    let start = NaiveDate::from_ymd_opt(y, 1, 1)?;
                    let end = NaiveDate::from_ymd_opt(y.checked_add(1)?, 1, 1)?;
                    return span_window(start, end, offset);
                }
            }
        }
    }

    None
}

fn day_window(day: NaiveDate, offset: FixedOffset) -> Option<TimeWindow> {
    span_window(day, day.succ_opt()?, offset)
}

fn span_window(start_day: NaiveDate, end_day: NaiveDate, offset: FixedOffset) -> Option<TimeWindow> {
    let naive_start = start_day.and_hms_opt(0, 0, 0)?;
    let naive_end = end_day.and_hms_opt(0, 0, 0)?;

    let local_start = offset.from_local_datetime(&naive_start).single()?;
    let local_end = offset.from_local_datetime(&naive_end).single()?;

    Some(TimeWindow {
        start: local_start.with_timezone(&Utc),
        end: local_end.with_timezone(&Utc),
    })
}
