use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique UI state node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNode {
    pub id: String,
    pub url: String,
    pub snapshot: String,
    pub title: String,
}

/// Selector information for an element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorInfo {
    pub raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aria: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Element information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A directed edge representing an action between states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub selector: SelectorInfo,
    pub element: ElementInfo,
    pub action_type: String,
    pub input_key: Option<String>,
    pub description: String,
}

/// A task reference in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRef {
    pub name: String,
    pub start_node: String,
    pub end_node: String,
}

/// The complete action map output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMap {
    pub site: String,
    pub generated_at: String,
    pub nodes: HashMap<String, StateNode>,
    pub edges: Vec<ActionEdge>,
    pub tasks: Vec<TaskRef>,
}

/// State held in the daemon during mapping
pub struct MapperState {
    pub active: bool,
    pub graph: super::StateGraph,
    pub site: String,
    pub current_state_id: Option<String>,
    pub task_name: Option<String>,
    pub task_start_node: Option<String>,
    pub tasks: Vec<TaskRef>,
}

impl MapperState {
    pub fn new() -> Self {
        Self {
            active: false,
            graph: super::StateGraph::new(),
            site: String::new(),
            current_state_id: None,
            task_name: None,
            task_start_node: None,
            tasks: Vec::new(),
        }
    }
}
