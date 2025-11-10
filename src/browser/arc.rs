use super::{BrowserExtractor, BrowserHistory, BrowserType};
use super::chrome::ChromiumBasedExtractor;
use anyhow::Result;
use std::path::PathBuf;

pub struct ArcExtractor {
    inner: ChromiumBasedExtractor,
}

impl ArcExtractor {
    pub fn new() -> Self {
        Self {
            inner: ChromiumBasedExtractor::new(BrowserType::Arc, "Arc/User Data"),
        }
    }
}

impl BrowserExtractor for ArcExtractor {
    fn extract(&self) -> Result<Vec<BrowserHistory>> {
        self.inner.extract()
    }

    fn get_browser_type(&self) -> BrowserType {
        BrowserType::Arc
    }

    fn get_history_paths(&self) -> Vec<PathBuf> {
        self.inner.get_history_paths()
    }
}