use super::graph::StateGraph;
use super::types::{ActionMap, TaskRef};
use std::path::Path;

/// Serialize the state graph and task references to JSON file
pub fn save_action_map(
    graph: &StateGraph,
    tasks: Vec<TaskRef>,
    site: &str,
    output_path: &Path,
) -> Result<(), String> {
    let generated_at = chrono::Utc::now().to_rfc3339();

    let action_map = ActionMap {
        site: site.to_string(),
        generated_at,
        nodes: graph.nodes().clone(),
        edges: graph.edges().clone(),
        tasks,
    };

    let json = serde_json::to_string_pretty(&action_map)
        .map_err(|e| format!("Failed to serialize action map: {}", e))?;

    std::fs::write(output_path, json)
        .map_err(|e| format!("Failed to write action map to file: {}", e))?;

    Ok(())
}
