use super::{BrowserExtractor, BrowserHistory, BrowserType, HistoryEntry};
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

pub struct ChromeExtractor {
    browser_type: BrowserType,
    app_support_path: String,
}

impl ChromeExtractor {
    pub fn new() -> Self {
        Self {
            browser_type: BrowserType::Chrome,
            app_support_path: "Google/Chrome".to_string(),
        }
    }
}

pub struct ChromiumBasedExtractor {
    browser_type: BrowserType,
    app_support_path: String,
}

impl ChromiumBasedExtractor {
    pub fn new(browser_type: BrowserType, app_support_path: &str) -> Self {
        Self {
            browser_type,
            app_support_path: app_support_path.to_string(),
        }
    }
}

impl BrowserExtractor for ChromeExtractor {
    fn extract(&self) -> Result<Vec<BrowserHistory>> {
        let extractor = ChromiumBasedExtractor::new(self.browser_type.clone(), &self.app_support_path);
        extractor.extract()
    }

    fn get_browser_type(&self) -> BrowserType {
        self.browser_type.clone()
    }

    fn get_history_paths(&self) -> Vec<PathBuf> {
        let extractor = ChromiumBasedExtractor::new(self.browser_type.clone(), &self.app_support_path);
        extractor.get_history_paths()
    }
}

impl BrowserExtractor for ChromiumBasedExtractor {
    fn extract(&self) -> Result<Vec<BrowserHistory>> {
        let mut all_histories = Vec::new();
        let paths = self.get_history_paths();

        for path in paths {
            if !path.exists() {
                continue;
            }

            let profile = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("Default")
                .to_string();

            match extract_chromium_history(&path, self.browser_type.clone(), profile.clone()) {
                Ok(history) => all_histories.push(history),
                Err(e) => {
                    tracing::warn!("Failed to extract from {:?} profile {}: {}", self.browser_type, profile, e);
                }
            }
        }

        Ok(all_histories)
    }

    fn get_browser_type(&self) -> BrowserType {
        self.browser_type.clone()
    }

    fn get_history_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let home = home::home_dir().expect("Could not find home directory");
        let base = home.join("Library/Application Support").join(&self.app_support_path);

        if !base.exists() {
            return paths;
        }

        // Check common profile directories
        for profile in ["Default", "Profile 1", "Profile 2", "Profile 3", "Profile 4", "Profile 5"] {
            let history = base.join(profile).join("History");
            if history.exists() {
                paths.push(history);
            }
        }

        // If no profiles found, check if History exists directly
        if paths.is_empty() {
            let default_history = base.join("History");
            if default_history.exists() {
                paths.push(default_history);
            }
        }

        paths
    }
}

fn extract_chromium_history(
    path: &Path,
    browser_type: BrowserType,
    profile: String,
) -> Result<BrowserHistory> {
    // Copy the history file to temp location to avoid locking issues
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("browser_history_temp_{}.db", uuid::Uuid::new_v4()));
    std::fs::copy(path, &temp_file)
        .with_context(|| format!("Failed to copy history file from {:?}", path))?;

    let conn = Connection::open_with_flags(
        &temp_file,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .with_context(|| format!("Failed to open database at {:?}", temp_file))?;

    let mut stmt = conn.prepare(
        "SELECT
            id,
            url,
            title,
            visit_count,
            typed_count,
            last_visit_time,
            hidden
        FROM urls
        ORDER BY last_visit_time DESC"
    )?;

    let entries: Result<Vec<HistoryEntry>> = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let url: String = row.get(1)?;
            let title: Option<String> = row.get(2)?;
            let visit_count: i32 = row.get(3)?;
            let typed_count: i32 = row.get(4)?;
            let last_visit_time_webkit: i64 = row.get(5)?;
            let hidden: i32 = row.get(6)?;

            // Chrome uses WebKit timestamp (microseconds since 1601-01-01)
            let last_visit_time = webkit_to_datetime(last_visit_time_webkit);

            Ok(HistoryEntry {
                id: id.to_string(),
                url,
                title,
                visit_time: last_visit_time,
                visit_count,
                typed_count,
                last_visit_time,
                hidden: hidden != 0,
                favicon_url: None,
            })
        })?
        .collect();

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    Ok(BrowserHistory {
        browser: browser_type,
        profile,
        entries: entries?,
    })
}

// Convert WebKit timestamp to DateTime<Utc>
// WebKit timestamp is microseconds since January 1, 1601 UTC
fn webkit_to_datetime(webkit_timestamp: i64) -> DateTime<Utc> {
    const WEBKIT_EPOCH_OFFSET: i64 = 11644473600; // Seconds between 1601-01-01 and 1970-01-01

    let seconds_since_unix = (webkit_timestamp / 1_000_000) - WEBKIT_EPOCH_OFFSET;
    let nanos = ((webkit_timestamp % 1_000_000) * 1000) as u32;

    DateTime::from_timestamp(seconds_since_unix, nanos)
        .unwrap_or_else(|| Utc::now())
}

// Add uuid to Cargo.toml dependencies
pub fn add_uuid_dependency() -> &'static str {
    r#"uuid = { version = "1.6", features = ["v4"] }"#
}