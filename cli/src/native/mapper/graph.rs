use super::types::{ActionEdge, StateNode};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// State graph manager with deduplication
pub struct StateGraph {
    nodes: HashMap<String, StateNode>,
    edges: Vec<ActionEdge>,
    edge_counter: usize,
}

impl StateGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            edge_counter: 0,
        }
    }

    /// Compute a stable hash for a state based on URL pattern and structural ARIA snapshot
    /// This method aggressively generalizes to map similar pages to the same state
    pub fn compute_state_hash(url: &str, snapshot: &str) -> String {
        // Get URL pattern (not just normalized, but categorized by page type)
        let url_pattern = Self::get_url_pattern(url);

        // Extract structural patterns only - roles and types, not specific text
        let structural_patterns: Vec<String> = snapshot
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
                }

                // Extract just the role/element type
                let parts: Vec<&str> = trimmed.split('"').collect();
                if let Some(first_part) = parts.first() {
                    let role = first_part
                        .trim()
                        .trim_start_matches('-')
                        .trim()
                        .split_whitespace()
                        .next()?;

                    // Normalize all interactive elements to their base type
                    // Ignore transient attributes like disabled, expanded, etc.
                    match role {
                        "link" => Some("link".to_string()),
                        "button" => Some("button".to_string()),
                        "textbox" | "searchbox" => Some("input".to_string()),
                        "combobox" => Some("select".to_string()),
                        "heading" => {
                            // Extract heading level for structure
                            if first_part.contains("[level=") {
                                let level = first_part
                                    .split("[level=")
                                    .nth(1)?
                                    .chars()
                                    .next()?;
                                Some(format!("h{}", level))
                            } else {
                                Some("heading".to_string())
                            }
                        }
                        "navigation" => Some("nav".to_string()),
                        "listitem" => Some("li".to_string()),
                        "generic" => None, // Skip generic containers
                        _ => Some(role.to_string()),
                    }
                } else {
                    None
                }
            })
            .collect();

        // Count occurrences of each element type
        let mut element_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for pattern in &structural_patterns {
            *element_counts.entry(pattern.clone()).or_insert(0) += 1;
        }

        // Group counts into buckets for more generalization
        // Instead of exact counts, use ranges: 1, 2-5, 6-10, 11-20, 20+
        let bucket_count = |count: usize| -> String {
            if count == 1 {
                "1".to_string()
            } else if count <= 5 {
                "2-5".to_string()
            } else if count <= 10 {
                "6-10".to_string()
            } else if count <= 20 {
                "11-20".to_string()
            } else if count <= 50 {
                "21-50".to_string()
            } else {
                "50+".to_string()
            }
        };

        // Create a canonical representation with bucketed counts
        let mut canonical_parts: Vec<String> = element_counts
            .iter()
            .map(|(element, count)| format!("{}:{}", element, bucket_count(*count)))
            .collect();
        canonical_parts.sort();
        let canonical_structure = canonical_parts.join(",");

        // Hash URL pattern + canonical structure
        let mut hasher = Sha256::new();
        hasher.update(url_pattern.as_bytes());
        hasher.update(b"\n");
        hasher.update(canonical_structure.as_bytes());
        let hash = hasher.finalize();

        // Use first 16 hex chars as state ID
        format!("{:x}", hash)[..16].to_string()
    }

    /// Get URL pattern - categorizes URLs by their type rather than specific content
    fn get_url_pattern(url: &str) -> String {
        use url::Url;

        if let Ok(parsed) = Url::parse(url) {
            let host = parsed.host_str().unwrap_or("unknown");
            let path = parsed.path();

            // Detect common page patterns
            match host {
                h if h.contains("github.com") => {
                    if path == "/" || path.is_empty() {
                        "github:home".to_string()
                    } else if path.matches('/').count() == 2 {
                        // /owner/repo
                        "github:repo".to_string()
                    } else if path.contains("/issues") && !path.contains("/issues/") {
                        // /owner/repo/issues
                        "github:issues_list".to_string()
                    } else if path.contains("/issues/") {
                        // /owner/repo/issues/123
                        "github:issue_detail".to_string()
                    } else if path.contains("/pull/") {
                        "github:pr_detail".to_string()
                    } else if path.contains("/pulls") {
                        "github:pulls_list".to_string()
                    } else if path.contains("/commits") {
                        "github:commits".to_string()
                    } else {
                        format!("github:other:{}", Self::normalize_path(path))
                    }
                }
                h if h.contains("reddit.com") => {
                    if path.starts_with("/r/") {
                        if path.matches('/').count() == 2 {
                            // /r/subreddit
                            "reddit:subreddit".to_string()
                        } else if path.contains("/comments/") {
                            "reddit:post".to_string()
                        } else {
                            "reddit:subreddit_other".to_string()
                        }
                    } else {
                        "reddit:other".to_string()
                    }
                }
                h if h.contains("twitter.com") || h.contains("x.com") => {
                    if path.matches('/').count() == 1 && path != "/" {
                        // /username
                        "twitter:profile".to_string()
                    } else if path.contains("/status/") {
                        "twitter:tweet".to_string()
                    } else {
                        "twitter:other".to_string()
                    }
                }
                _ => {
                    // Generic pattern: host + path structure
                    format!("{}:{}", host, Self::normalize_path(path))
                }
            }
        } else {
            url.to_string()
        }
    }

    /// Normalize path by replacing IDs with placeholders
    fn normalize_path(path: &str) -> String {
        path.split('/')
            .map(|segment| {
                if segment.is_empty() {
                    "".to_string()
                } else if segment.chars().all(|c| c.is_ascii_digit()) {
                    "*".to_string()
                } else if segment.len() > 20 && segment.contains('-') {
                    "*".to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    /// Normalize URL to remove dynamic parts
    fn normalize_url(url: &str) -> String {
        use url::Url;

        if let Ok(parsed) = Url::parse(url) {
            let mut normalized = String::new();
            normalized.push_str(parsed.scheme());
            normalized.push_str("://");
            if let Some(host) = parsed.host_str() {
                normalized.push_str(host);
            }

            // Keep path but normalize patterns like /post/123 to /post/*
            let path = parsed.path();
            let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            let normalized_segments: Vec<String> = path_segments
                .iter()
                .map(|segment| {
                    // If segment looks like an ID (all digits, UUID, etc.), replace with *
                    if !segment.is_empty() && segment.chars().all(|c| c.is_ascii_digit()) {
                        "*".to_string()
                    } else if segment.len() > 20 && segment.contains('-') && segment.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                        // Looks like a UUID or long ID
                        "*".to_string()
                    } else {
                        segment.to_string()
                    }
                })
                .collect();

            if !normalized_segments.is_empty() {
                normalized.push('/');
                normalized.push_str(&normalized_segments.join("/"));
            }
            normalized
        } else {
            url.to_string()
        }
    }

    /// Get or insert a state node, returns the node ID
    pub fn get_or_insert_node(&mut self, url: &str, snapshot: &str, title: &str) -> String {
        let node_id = Self::compute_state_hash(url, snapshot);

        if !self.nodes.contains_key(&node_id) {
            let node = StateNode {
                id: node_id.clone(),
                url: url.to_string(),
                snapshot: snapshot.to_string(),
                title: title.to_string(),
            };
            self.nodes.insert(node_id.clone(), node);
        }

        node_id
    }

    /// Add an edge between two states
    pub fn add_edge(
        &mut self,
        from: &str,
        to: &str,
        selector: super::types::SelectorInfo,
        element: super::types::ElementInfo,
        action_type: &str,
        input_key: Option<&str>,
        description: &str,
    ) -> String {
        self.edge_counter += 1;
        let edge_id = format!("e{}", self.edge_counter);

        let edge = ActionEdge {
            id: edge_id.clone(),
            from: from.to_string(),
            to: to.to_string(),
            selector,
            element,
            action_type: action_type.to_string(),
            input_key: input_key.map(|s| s.to_string()),
            description: description.to_string(),
        };

        self.edges.push(edge);
        edge_id
    }

    /// Get all nodes
    pub fn nodes(&self) -> &HashMap<String, StateNode> {
        &self.nodes
    }

    /// Get all edges
    pub fn edges(&self) -> &Vec<ActionEdge> {
        &self.edges
    }

    /// Check if a node exists
    pub fn has_node(&self, node_id: &str) -> bool {
        self.nodes.contains_key(node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::mapper::types::{ElementInfo, SelectorInfo};

    #[test]
    fn test_compute_state_hash_stable() {
        let url = "https://example.com";
        let snapshot1 = "- button \"Submit\" [ref=e1]\n- textbox \"Email\" [ref=e2]";
        let snapshot2 = "- button \"Cancel\" [ref=e5]\n- textbox \"Email\" [ref=e6]";

        // Same structure (button + textbox), different text/refs -> same hash
        let hash1 = StateGraph::compute_state_hash(url, snapshot1);
        let hash2 = StateGraph::compute_state_hash(url, snapshot2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_state_hash_reddit_subreddits() {
        let url1 = "https://reddit.com/r/rust";
        let url2 = "https://reddit.com/r/programming";

        // Different subreddits but same structure
        let snapshot1 = "- link \"Post 1\" [ref=e1]\n- link \"Post 2\" [ref=e2]\n- button \"Upvote\" [ref=e3]\n- button \"Upvote\" [ref=e4]";
        let snapshot2 = "- link \"Other post\" [ref=e5]\n- link \"Another\" [ref=e6]\n- button \"Upvote\" [ref=e7]\n- button \"Upvote\" [ref=e8]";

        // Same page type (reddit:subreddit) + same structure -> same hash
        let hash1 = StateGraph::compute_state_hash(url1, snapshot1);
        let hash2 = StateGraph::compute_state_hash(url2, snapshot2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_state_hash_github_repos() {
        let url1 = "https://github.com/rust-lang/rust";
        let url2 = "https://github.com/facebook/react";

        // Different repos but same structure
        let snapshot = "- button \"Code\" [ref=e1]\n- link \"Issues\" [ref=e2]\n- heading \"README\" [ref=e3]";

        // Same page type (github:repo) + same structure -> same hash
        let hash1 = StateGraph::compute_state_hash(url1, snapshot);
        let hash2 = StateGraph::compute_state_hash(url2, snapshot);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_state_hash_count_buckets() {
        let url = "https://reddit.com/r/rust";

        // 3 links and 4 links should map to same bucket (2-5)
        let snapshot1 = "- link \"1\" [ref=e1]\n- link \"2\" [ref=e2]\n- link \"3\" [ref=e3]";
        let snapshot2 = "- link \"1\" [ref=e1]\n- link \"2\" [ref=e2]\n- link \"3\" [ref=e3]\n- link \"4\" [ref=e4]";

        let hash1 = StateGraph::compute_state_hash(url, snapshot1);
        let hash2 = StateGraph::compute_state_hash(url, snapshot2);

        assert_eq!(hash1, hash2, "3 and 4 links should map to same bucket");

        // But 3 links and 10 links should be different buckets
        let snapshot3 = "- link \"1\" [ref=e1]\n- link \"2\" [ref=e2]\n- link \"3\" [ref=e3]\n- link \"4\" [ref=e4]\n- link \"5\" [ref=e5]\n- link \"6\" [ref=e6]\n- link \"7\" [ref=e7]\n- link \"8\" [ref=e8]\n- link \"9\" [ref=e9]\n- link \"10\" [ref=e10]";

        let hash3 = StateGraph::compute_state_hash(url, snapshot3);
        assert_ne!(hash1, hash3, "3 links and 10 links should be different buckets");
    }

    #[test]
    fn test_get_url_pattern() {
        assert_eq!(
            StateGraph::get_url_pattern("https://github.com/rust-lang/rust"),
            "github:repo"
        );
        assert_eq!(
            StateGraph::get_url_pattern("https://github.com/facebook/react"),
            "github:repo"
        );
        assert_eq!(
            StateGraph::get_url_pattern("https://github.com/rust-lang/rust/issues"),
            "github:issues_list"
        );
        assert_eq!(
            StateGraph::get_url_pattern("https://github.com/rust-lang/rust/issues/12345"),
            "github:issue_detail"
        );
        assert_eq!(
            StateGraph::get_url_pattern("https://reddit.com/r/rust"),
            "reddit:subreddit"
        );
        assert_eq!(
            StateGraph::get_url_pattern("https://reddit.com/r/programming"),
            "reddit:subreddit"
        );
    }

    #[test]
    fn test_get_or_insert_node() {
        let mut graph = StateGraph::new();
        let url = "https://example.com";
        let snapshot = "- button \"Submit\" [ref=e1]";
        let title = "Example";

        let id1 = graph.get_or_insert_node(url, snapshot, title);
        let id2 = graph.get_or_insert_node(url, snapshot, title);

        assert_eq!(id1, id2);
        assert_eq!(graph.nodes().len(), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = StateGraph::new();

        let from = "state1";
        let to = "state2";
        let selector = SelectorInfo {
            raw: "#submit".to_string(),
            aria: Some("role=button name=\"Submit\"".to_string()),
            name: Some("Submit".to_string()),
            role: Some("button".to_string()),
        };
        let element = ElementInfo {
            tag: Some("button".to_string()),
            class: Some("btn-primary".to_string()),
            id: Some("submit".to_string()),
        };
        let edge_id = graph.add_edge(from, to, selector, element, "click", None, "Click submit");

        assert!(!edge_id.is_empty());
        assert_eq!(graph.edges().len(), 1);
        assert_eq!(graph.edges()[0].from, "state1");
        assert_eq!(graph.edges()[0].to, "state2");
    }
}
