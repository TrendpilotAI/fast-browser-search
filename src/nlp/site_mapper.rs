use once_cell::sync::Lazy;
use std::collections::HashMap;
use url::Url;

/// Maps domain names to friendly display names
static SITE_NAME_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Developer & Tech
    map.insert("github.com", "GitHub");
    map.insert("gitlab.com", "GitLab");
    map.insert("bitbucket.org", "Bitbucket");
    map.insert("stackoverflow.com", "Stack Overflow");
    map.insert("docs.rust-lang.org", "Rust Documentation");
    map.insert("crates.io", "Crates.io");
    map.insert("npmjs.com", "npm");
    map.insert("pypi.org", "PyPI");
    map.insert("hackernews.com", "Hacker News");
    map.insert("news.ycombinator.com", "Hacker News");
    map.insert("dev.to", "DEV Community");
    map.insert("medium.com", "Medium");
    map.insert("hashnode.com", "Hashnode");

    // AI & ML
    map.insert("claude.ai", "Claude");
    map.insert("chat.openai.com", "ChatGPT");
    map.insert("bard.google.com", "Google Bard");
    map.insert("perplexity.ai", "Perplexity");
    map.insert("huggingface.co", "Hugging Face");
    map.insert("kaggle.com", "Kaggle");
    map.insert("colab.research.google.com", "Google Colab");

    // Social Media
    map.insert("reddit.com", "Reddit");
    map.insert("twitter.com", "Twitter");
    map.insert("x.com", "X (Twitter)");
    map.insert("facebook.com", "Facebook");
    map.insert("instagram.com", "Instagram");
    map.insert("linkedin.com", "LinkedIn");
    map.insert("youtube.com", "YouTube");
    map.insert("tiktok.com", "TikTok");
    map.insert("discord.com", "Discord");
    map.insert("slack.com", "Slack");
    map.insert("telegram.org", "Telegram");

    // Productivity & Work
    map.insert("notion.so", "Notion");
    map.insert("todoist.com", "Todoist");
    map.insert("trello.com", "Trello");
    map.insert("asana.com", "Asana");
    map.insert("monday.com", "Monday");
    map.insert("figma.com", "Figma");
    map.insert("miro.com", "Miro");
    map.insert("canva.com", "Canva");

    // Documentation & Learning
    map.insert("wikipedia.org", "Wikipedia");
    map.insert("coursera.org", "Coursera");
    map.insert("udemy.com", "Udemy");
    map.insert("edx.org", "edX");
    map.insert("khanacademy.org", "Khan Academy");
    map.insert("arxiv.org", "arXiv");
    map.insert("scholar.google.com", "Google Scholar");

    // Cloud & Services
    map.insert("aws.amazon.com", "AWS");
    map.insert("cloud.google.com", "Google Cloud");
    map.insert("azure.microsoft.com", "Azure");
    map.insert("vercel.com", "Vercel");
    map.insert("netlify.com", "Netlify");
    map.insert("heroku.com", "Heroku");
    map.insert("digitalocean.com", "DigitalOcean");

    // E-commerce
    map.insert("amazon.com", "Amazon");
    map.insert("ebay.com", "eBay");
    map.insert("shopify.com", "Shopify");
    map.insert("etsy.com", "Etsy");
    map.insert("alibaba.com", "Alibaba");

    // News & Media
    map.insert("nytimes.com", "The New York Times");
    map.insert("wsj.com", "Wall Street Journal");
    map.insert("bbc.com", "BBC");
    map.insert("cnn.com", "CNN");
    map.insert("reuters.com", "Reuters");
    map.insert("theguardian.com", "The Guardian");
    map.insert("washingtonpost.com", "Washington Post");

    // Search Engines
    map.insert("google.com", "Google");
    map.insert("bing.com", "Bing");
    map.insert("duckduckgo.com", "DuckDuckGo");
    map.insert("baidu.com", "Baidu");

    // Email
    map.insert("gmail.com", "Gmail");
    map.insert("outlook.com", "Outlook");
    map.insert("mail.google.com", "Gmail");
    map.insert("outlook.live.com", "Outlook");

    map
});

/// Categories for different site types
static SITE_CATEGORIES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Developer & Tech
    map.insert("github.com", "Developer Tools");
    map.insert("stackoverflow.com", "Developer Tools");
    map.insert("docs.rust-lang.org", "Documentation");
    map.insert("crates.io", "Package Registry");

    // AI & ML
    map.insert("claude.ai", "AI Assistant");
    map.insert("chat.openai.com", "AI Assistant");
    map.insert("huggingface.co", "AI/ML Platform");

    // Social Media
    map.insert("reddit.com", "Social Media");
    map.insert("twitter.com", "Social Media");
    map.insert("youtube.com", "Video Platform");

    // Productivity
    map.insert("notion.so", "Productivity");
    map.insert("figma.com", "Design Tool");

    // Learning
    map.insert("wikipedia.org", "Reference");
    map.insert("coursera.org", "Online Learning");

    map
});

pub struct SiteMapper;

impl SiteMapper {
    /// Get a beautified name for a URL
    pub fn get_clean_site_name(url_str: &str) -> String {
        if let Ok(url) = Url::parse(url_str) {
            if let Some(host) = url.host_str() {
                // Remove www. prefix
                let clean_host = host.strip_prefix("www.").unwrap_or(host);

                // Check if we have a custom name
                if let Some(name) = SITE_NAME_MAP.get(clean_host) {
                    return (*name).to_string();
                }

                // For subdomains, try to create a nice name
                let parts: Vec<&str> = clean_host.split('.').collect();
                if parts.len() > 2 {
                    // e.g., docs.rust-lang.org -> "Rust Docs"
                    if let Some(base_name) = SITE_NAME_MAP.get(parts[1..].join(".").as_str()) {
                        let subdomain = capitalize_first(parts[0]);
                        return format!("{} {}", base_name, subdomain);
                    }
                }

                // Default: capitalize domain name without TLD
                if let Some(domain) = parts.first() {
                    return capitalize_words(domain.replace('-', " ").replace('_', " ").as_str());
                }

                return clean_host.to_string();
            }
        }

        // Fallback: return the original URL
        url_str.to_string()
    }

    /// Get the category for a URL
    pub fn get_site_category(url_str: &str) -> Option<String> {
        if let Ok(url) = Url::parse(url_str) {
            if let Some(host) = url.host_str() {
                let clean_host = host.strip_prefix("www.").unwrap_or(host);

                if let Some(category) = SITE_CATEGORIES.get(clean_host) {
                    return Some((*category).to_string());
                }

                // Try to infer category from domain
                if clean_host.ends_with(".edu") {
                    return Some("Education".to_string());
                }
                if clean_host.ends_with(".gov") {
                    return Some("Government".to_string());
                }
                if clean_host.ends_with(".org") {
                    return Some("Organization".to_string());
                }
                if clean_host.contains("blog") || clean_host.contains("news") {
                    return Some("Blog/News".to_string());
                }
                if clean_host.contains("docs") || clean_host.contains("documentation") {
                    return Some("Documentation".to_string());
                }
                if clean_host.contains("api") {
                    return Some("API".to_string());
                }
            }
        }
        None
    }

    /// Extract domain and subdomain from URL
    pub fn extract_domain_parts(url_str: &str) -> (String, Option<String>) {
        if let Ok(url) = Url::parse(url_str) {
            if let Some(host) = url.host_str() {
                let clean_host = host.strip_prefix("www.").unwrap_or(host);
                let parts: Vec<&str> = clean_host.split('.').collect();

                if parts.len() > 2 {
                    let subdomain = parts[0].to_string();
                    let domain = parts[1..].join(".");
                    return (domain, Some(subdomain));
                } else {
                    return (clean_host.to_string(), None);
                }
            }
        }
        (url_str.to_string(), None)
    }

    /// Extract path segments from URL
    pub fn extract_path_segments(url_str: &str) -> Vec<String> {
        if let Ok(url) = Url::parse(url_str) {
            url.path_segments()
                .map(|segments| {
                    segments
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }
}

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Capitalize each word in a string
fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(capitalize_first)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_name_mapping() {
        assert_eq!(SiteMapper::get_clean_site_name("https://github.com/user/repo"), "GitHub");
        assert_eq!(SiteMapper::get_clean_site_name("https://www.reddit.com/r/rust"), "Reddit");
        assert_eq!(SiteMapper::get_clean_site_name("https://docs.rust-lang.org/book"), "Rust Documentation");
        assert_eq!(SiteMapper::get_clean_site_name("https://claude.ai/chat/123"), "Claude");
    }

    #[test]
    fn test_category_extraction() {
        assert_eq!(SiteMapper::get_site_category("https://github.com"), Some("Developer Tools".to_string()));
        assert_eq!(SiteMapper::get_site_category("https://claude.ai"), Some("AI Assistant".to_string()));
        assert_eq!(SiteMapper::get_site_category("https://university.edu"), Some("Education".to_string()));
    }

    #[test]
    fn test_domain_extraction() {
        let (domain, subdomain) = SiteMapper::extract_domain_parts("https://docs.rust-lang.org");
        assert_eq!(domain, "rust-lang.org");
        assert_eq!(subdomain, Some("docs".to_string()));

        let (domain, subdomain) = SiteMapper::extract_domain_parts("https://github.com");
        assert_eq!(domain, "github.com");
        assert_eq!(subdomain, None);
    }

    #[test]
    fn test_path_extraction() {
        let segments = SiteMapper::extract_path_segments("https://github.com/user/repo/blob/main/README.md");
        assert_eq!(segments, vec!["user", "repo", "blob", "main", "README.md"]);
    }
}