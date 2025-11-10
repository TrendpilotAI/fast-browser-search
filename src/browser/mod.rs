use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub mod chrome;
pub mod safari;
pub mod arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserHistory {
    pub browser: BrowserType,
    pub profile: String,
    pub entries: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Safari,
    Arc,
    Comet,
    Genspark,
    Thorium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub visit_time: DateTime<Utc>,
    pub visit_count: i32,
    pub typed_count: i32,
    pub last_visit_time: DateTime<Utc>,
    pub hidden: bool,
    pub favicon_url: Option<String>,
}

pub trait BrowserExtractor: Send + Sync {
    fn extract(&self) -> Result<Vec<BrowserHistory>>;
    fn get_browser_type(&self) -> BrowserType;
    fn get_history_paths(&self) -> Vec<PathBuf>;
}

pub fn get_all_browser_histories() -> Result<Vec<BrowserHistory>> {
    let mut all_histories = Vec::new();

    // Extract from each browser
    let extractors: Vec<Box<dyn BrowserExtractor>> = vec![
        Box::new(chrome::ChromeExtractor::new()),
        Box::new(safari::SafariExtractor::new()),
        Box::new(arc::ArcExtractor::new()),
        Box::new(chrome::ChromiumBasedExtractor::new(BrowserType::Comet, "Comet")),
        Box::new(chrome::ChromiumBasedExtractor::new(BrowserType::Genspark, "GensparkSoftware/Genspark-Browser")),
        Box::new(chrome::ChromiumBasedExtractor::new(BrowserType::Thorium, "Thorium")),
    ];

    for extractor in extractors {
        match extractor.extract() {
            Ok(histories) => {
                tracing::info!("Extracted {} history sets from {:?}", histories.len(), extractor.get_browser_type());
                all_histories.extend(histories);
            }
            Err(e) => {
                tracing::warn!("Failed to extract from {:?}: {}", extractor.get_browser_type(), e);
            }
        }
    }

    Ok(all_histories)
}

pub fn find_browser_history_files() -> Vec<(BrowserType, PathBuf)> {
    let mut found_files = Vec::new();
    let home = home::home_dir().expect("Could not find home directory");
    let library = home.join("Library/Application Support");

    // Chrome profiles
    let chrome_base = library.join("Google/Chrome");
    if chrome_base.exists() {
        for profile in ["Default", "Profile 1", "Profile 2", "Profile 3"] {
            let history = chrome_base.join(profile).join("History");
            if history.exists() {
                found_files.push((BrowserType::Chrome, history));
            }
        }
    }

    // Safari
    let safari_history = home.join("Library/Safari/History.db");
    if safari_history.exists() {
        found_files.push((BrowserType::Safari, safari_history));
    }

    // Arc
    let arc_history = library.join("Arc/User Data/Default/History");
    if arc_history.exists() {
        found_files.push((BrowserType::Arc, arc_history));
    }

    // Comet
    let comet_history = library.join("Comet/Default/History");
    if comet_history.exists() {
        found_files.push((BrowserType::Comet, comet_history));
    }

    // Genspark
    let genspark_history = library.join("GensparkSoftware/Genspark-Browser/Default/History");
    if genspark_history.exists() {
        found_files.push((BrowserType::Genspark, genspark_history));
    }

    // Thorium
    let thorium_base = library.join("Thorium");
    if thorium_base.exists() {
        for profile in ["Default", "Profile 1", "Profile 2"] {
            let history = thorium_base.join(profile).join("History");
            if history.exists() {
                found_files.push((BrowserType::Thorium, history));
            }
        }
    }

    found_files
}