use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

use crate::browser::HistoryEntry;

/// Summarizes browsing history and generates insights
pub struct Summarizer;

impl Summarizer {
    /// Generate a summary for a single entry
    pub fn summarize_entry(title: Option<&str>, url: &str, keywords: &[String]) -> String {
        if let Some(title) = title {
            // If we have a title, use it as the base summary
            if title.len() > 100 {
                // Truncate long titles
                format!("{}...", &title[..97])
            } else {
                title.to_string()
            }
        } else {
            // Generate summary from URL and keywords
            let domain = url.split('/').nth(2).unwrap_or(url);
            if !keywords.is_empty() {
                format!("Page about {} on {}", keywords.join(", "), domain)
            } else {
                format!("Visit to {}", domain)
            }
        }
    }

    /// Group entries into browsing sessions
    pub fn group_into_sessions(
        entries: &[HistoryEntry],
        session_gap_minutes: i64,
    ) -> Vec<BrowsingSession> {
        if entries.is_empty() {
            return Vec::new();
        }

        let mut sessions = Vec::new();
        let mut current_session = BrowsingSession::new();
        let mut last_time: Option<DateTime<Utc>> = None;

        for entry in entries {
            let should_start_new_session = if let Some(last) = last_time {
                entry.visit_time - last > Duration::minutes(session_gap_minutes)
            } else {
                false
            };

            if should_start_new_session && !current_session.entries.is_empty() {
                sessions.push(current_session);
                current_session = BrowsingSession::new();
            }

            current_session.add_entry(entry.clone());
            last_time = Some(entry.visit_time);
        }

        if !current_session.entries.is_empty() {
            sessions.push(current_session);
        }

        sessions
    }

    /// Generate a session summary
    pub fn summarize_session(session: &BrowsingSession) -> SessionSummary {
        let domains = session.get_unique_domains();
        let duration = session.duration();
        let main_topics = session.extract_main_topics(5);

        let activity_type = if domains.len() == 1 {
            "Focused browsing"
        } else if main_topics.len() == 1 {
            "Topic research"
        } else if domains.contains(&"github.com".to_string()) || domains.contains(&"stackoverflow.com".to_string()) {
            "Development work"
        } else if domains.iter().any(|d| d.contains("news") || d.contains("reddit")) {
            "News reading"
        } else {
            "General browsing"
        };

        let description = format!(
            "{} session with {} pages across {} domains. Main topics: {}",
            activity_type,
            session.entries.len(),
            domains.len(),
            main_topics.join(", ")
        );

        SessionSummary {
            start_time: session.start_time(),
            end_time: session.end_time(),
            duration_minutes: duration.num_minutes(),
            page_count: session.entries.len(),
            unique_domains: domains.len(),
            main_topics,
            activity_type: activity_type.to_string(),
            description,
        }
    }

    /// Generate insights from browsing patterns
    pub fn generate_insights(entries: &[HistoryEntry]) -> BrowsingInsights {
        let mut domain_visits: HashMap<String, usize> = HashMap::new();
        let mut hourly_activity: HashMap<u32, usize> = HashMap::new();
        let mut daily_counts: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            // Count domain visits
            let domain = entry.url.split('/').nth(2).unwrap_or(&entry.url).to_string();
            *domain_visits.entry(domain).or_insert(0) += 1;

            // Count hourly activity
            let hour = entry.visit_time.format("%H").to_string().parse::<u32>().unwrap_or(0);
            *hourly_activity.entry(hour).or_insert(0) += 1;

            // Count daily activity
            let date = entry.visit_time.format("%Y-%m-%d").to_string();
            *daily_counts.entry(date).or_insert(0) += 1;
        }

        // Find most visited domains
        let mut domain_vec: Vec<(String, usize)> = domain_visits.into_iter().collect();
        domain_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let unique_domain_count = domain_vec.len();
        let top_domains: Vec<DomainStats> = domain_vec
            .into_iter()
            .take(10)
            .map(|(domain, count)| DomainStats {
                domain,
                visit_count: count,
                percentage: (count as f32 / entries.len() as f32) * 100.0,
            })
            .collect();

        // Find peak activity hours
        let peak_hour = hourly_activity
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(&hour, _)| hour)
            .unwrap_or(0);

        // Calculate average daily visits
        let avg_daily_visits = if !daily_counts.is_empty() {
            daily_counts.values().sum::<usize>() as f32 / daily_counts.len() as f32
        } else {
            0.0
        };

        BrowsingInsights {
            total_visits: entries.len(),
            unique_domains: unique_domain_count,
            top_domains,
            peak_activity_hour: peak_hour,
            average_daily_visits: avg_daily_visits,
            most_productive_day: daily_counts
                .iter()
                .max_by_key(|&(_, count)| count)
                .map(|(day, _)| day.clone()),
        }
    }
}

/// Represents a browsing session
#[derive(Debug, Clone)]
pub struct BrowsingSession {
    pub entries: Vec<HistoryEntry>,
}

impl BrowsingSession {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
    }

    pub fn start_time(&self) -> Option<DateTime<Utc>> {
        self.entries.first().map(|e| e.visit_time)
    }

    pub fn end_time(&self) -> Option<DateTime<Utc>> {
        self.entries.last().map(|e| e.visit_time)
    }

    pub fn duration(&self) -> Duration {
        if let (Some(start), Some(end)) = (self.start_time(), self.end_time()) {
            end - start
        } else {
            Duration::zero()
        }
    }

    pub fn get_unique_domains(&self) -> Vec<String> {
        let mut domains = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for entry in &self.entries {
            if let Some(domain) = entry.url.split('/').nth(2) {
                if seen.insert(domain.to_string()) {
                    domains.push(domain.to_string());
                }
            }
        }

        domains
    }

    pub fn extract_main_topics(&self, max_topics: usize) -> Vec<String> {
        let mut topic_counts: HashMap<String, usize> = HashMap::new();

        // Count word frequencies from titles
        for entry in &self.entries {
            if let Some(title) = &entry.title {
                for word in title.split_whitespace() {
                    let word_lower = word.to_lowercase();
                    if word_lower.len() > 3 {
                        *topic_counts.entry(word_lower).or_insert(0) += 1;
                    }
                }
            }
        }

        // Sort by frequency and take top topics
        let mut topics: Vec<(String, usize)> = topic_counts.into_iter().collect();
        topics.sort_by(|a, b| b.1.cmp(&a.1));

        topics
            .into_iter()
            .take(max_topics)
            .map(|(topic, _)| topic)
            .collect()
    }
}

/// Summary of a browsing session
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionSummary {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: i64,
    pub page_count: usize,
    pub unique_domains: usize,
    pub main_topics: Vec<String>,
    pub activity_type: String,
    pub description: String,
}

/// Insights from browsing patterns
#[derive(Debug, Clone, serde::Serialize)]
pub struct BrowsingInsights {
    pub total_visits: usize,
    pub unique_domains: usize,
    pub top_domains: Vec<DomainStats>,
    pub peak_activity_hour: u32,
    pub average_daily_visits: f32,
    pub most_productive_day: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DomainStats {
    pub domain: String,
    pub visit_count: usize,
    pub percentage: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(url: &str, title: &str, minutes_ago: i64) -> HistoryEntry {
        HistoryEntry {
            url: url.to_string(),
            title: Some(title.to_string()),
            visit_time: Utc::now() - Duration::minutes(minutes_ago),
            visit_count: 1,
        }
    }

    #[test]
    fn test_session_grouping() {
        let entries = vec![
            create_test_entry("https://github.com", "GitHub", 60),
            create_test_entry("https://github.com/rust", "Rust", 55),
            create_test_entry("https://stackoverflow.com", "Stack Overflow", 50),
            // Gap > 30 minutes
            create_test_entry("https://youtube.com", "YouTube", 10),
            create_test_entry("https://reddit.com", "Reddit", 5),
        ];

        let sessions = Summarizer::group_into_sessions(&entries, 30);

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].entries.len(), 3);
        assert_eq!(sessions[1].entries.len(), 2);
    }

    #[test]
    fn test_session_summary() {
        let mut session = BrowsingSession::new();
        session.add_entry(create_test_entry("https://github.com", "GitHub", 30));
        session.add_entry(create_test_entry("https://github.com/rust", "Rust Programming", 25));
        session.add_entry(create_test_entry("https://docs.rust-lang.org", "Rust Documentation", 20));

        let summary = Summarizer::summarize_session(&session);

        assert_eq!(summary.page_count, 3);
        assert_eq!(summary.unique_domains, 2);
        assert!(summary.activity_type.contains("Development") || summary.activity_type.contains("research"));
    }

    #[test]
    fn test_insights_generation() {
        let entries = vec![
            create_test_entry("https://github.com", "GitHub", 60),
            create_test_entry("https://github.com/rust", "Rust", 55),
            create_test_entry("https://stackoverflow.com", "Stack Overflow", 50),
            create_test_entry("https://github.com/project", "Project", 45),
        ];

        let insights = Summarizer::generate_insights(&entries);

        assert_eq!(insights.total_visits, 4);
        assert_eq!(insights.unique_domains, 2);
        assert!(!insights.top_domains.is_empty());
        assert_eq!(insights.top_domains[0].domain, "github.com");
        assert_eq!(insights.top_domains[0].visit_count, 3);
    }
}