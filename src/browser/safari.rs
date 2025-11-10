use super::{BrowserExtractor, BrowserHistory, BrowserType, HistoryEntry};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;

pub struct SafariExtractor {
    browser_type: BrowserType,
}

impl SafariExtractor {
    pub fn new() -> Self {
        Self {
            browser_type: BrowserType::Safari,
        }
    }
}

impl BrowserExtractor for SafariExtractor {
    fn extract(&self) -> Result<Vec<BrowserHistory>> {
        let paths = self.get_history_paths();
        let mut all_histories = Vec::new();

        for path in paths {
            if !path.exists() {
                continue;
            }

            match extract_safari_history(&path) {
                Ok(history) => all_histories.push(history),
                Err(e) => {
                    tracing::warn!("Failed to extract Safari history: {}", e);
                }
            }
        }

        Ok(all_histories)
    }

    fn get_browser_type(&self) -> BrowserType {
        self.browser_type.clone()
    }

    fn get_history_paths(&self) -> Vec<PathBuf> {
        let home = home::home_dir().expect("Could not find home directory");
        vec![home.join("Library/Safari/History.db")]
    }
}

fn extract_safari_history(path: &PathBuf) -> Result<BrowserHistory> {
    // Copy the history file to temp location to avoid permission issues
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("safari_history_temp_{}.db", uuid::Uuid::new_v4()));

    // Safari history may require elevated permissions, so we'll try to copy it
    std::fs::copy(path, &temp_file)
        .with_context(|| format!("Failed to copy Safari history from {:?}. May need elevated permissions.", path))?;

    let conn = Connection::open_with_flags(
        &temp_file,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .with_context(|| format!("Failed to open Safari database at {:?}", temp_file))?;

    // Safari uses different table structure
    let mut stmt = conn.prepare(
        "SELECT
            hi.id,
            hi.url,
            hv.title,
            hv.visit_time,
            hi.visit_count
        FROM history_items hi
        LEFT JOIN history_visits hv ON hi.id = hv.history_item
        WHERE hi.url IS NOT NULL
        ORDER BY hv.visit_time DESC"
    )?;

    let entries: Vec<HistoryEntry> = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let url: String = row.get(1)?;
            let title: Option<String> = row.get(2)?;
            let visit_time_cocoa: f64 = row.get(3)?;
            let visit_count: i32 = row.get(4)?;

            // Safari uses Core Data timestamp (seconds since 2001-01-01)
            let visit_time = cocoa_to_datetime(visit_time_cocoa);

            Ok(HistoryEntry {
                id: id.to_string(),
                url,
                title,
                visit_time,
                visit_count,
                typed_count: 0,
                last_visit_time: visit_time,
                hidden: false,
                favicon_url: None,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    Ok(BrowserHistory {
        browser: BrowserType::Safari,
        profile: "Default".to_string(),
        entries,
    })
}

// Convert Core Data timestamp to DateTime<Utc>
// Core Data timestamp is seconds since January 1, 2001 00:00:00 GMT
fn cocoa_to_datetime(cocoa_timestamp: f64) -> DateTime<Utc> {
    const COCOA_EPOCH_OFFSET: i64 = 978307200; // Seconds between 1970-01-01 and 2001-01-01

    let seconds_since_unix = cocoa_timestamp as i64 + COCOA_EPOCH_OFFSET;
    let nanos = (cocoa_timestamp.fract() * 1_000_000_000.0) as u32;

    DateTime::from_timestamp(seconds_since_unix, nanos)
        .unwrap_or_else(|| Utc::now())
}