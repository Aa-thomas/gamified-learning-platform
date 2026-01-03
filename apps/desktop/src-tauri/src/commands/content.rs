use crate::state::AppState;
use content::{ContentNode, Manifest, Quiz};
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct ContentTree {
    pub title: String,
    pub weeks: Vec<WeekData>,
}

#[derive(Serialize)]
pub struct WeekData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub days: Vec<DayData>,
}

#[derive(Serialize)]
pub struct DayData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub nodes: Vec<NodeData>,
}

#[derive(Serialize)]
pub struct NodeData {
    pub id: String,
    pub node_type: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub estimated_minutes: u32,
    pub xp_reward: u32,
    pub prerequisites: Vec<String>,
    pub skills: Vec<String>,
}

impl From<&ContentNode> for NodeData {
    fn from(node: &ContentNode) -> Self {
        Self {
            id: node.id.clone(),
            node_type: node.node_type.clone(),
            title: node.title.clone(),
            description: node.description.clone(),
            difficulty: node.difficulty.clone(),
            estimated_minutes: node.estimated_minutes,
            xp_reward: node.xp_reward,
            prerequisites: node.prerequisites.clone(),
            skills: node.skills.clone(),
        }
    }
}

impl From<&Manifest> for ContentTree {
    fn from(manifest: &Manifest) -> Self {
        Self {
            title: manifest.title.clone(),
            weeks: manifest
                .weeks
                .iter()
                .map(|w| WeekData {
                    id: w.id.clone(),
                    title: w.title.clone(),
                    description: w.description.clone(),
                    days: w
                        .days
                        .iter()
                        .map(|d| DayData {
                            id: d.id.clone(),
                            title: d.title.clone(),
                            description: d.description.clone(),
                            nodes: d.nodes.iter().map(NodeData::from).collect(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[tauri::command]
pub fn get_content_tree(state: State<AppState>) -> Result<Option<ContentTree>, String> {
    let loader = state.content_loader.lock().map_err(|e| e.to_string())?;

    match &*loader {
        Some(l) => Ok(Some(ContentTree::from(l.get_manifest()))),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_node_by_id(state: State<AppState>, node_id: String) -> Result<Option<NodeData>, String> {
    let loader = state.content_loader.lock().map_err(|e| e.to_string())?;

    match &*loader {
        Some(l) => Ok(l.get_node_by_id(&node_id).map(NodeData::from)),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn load_lecture(state: State<AppState>, content_path: String) -> Result<String, String> {
    let loader = state.content_loader.lock().map_err(|e| e.to_string())?;

    match &*loader {
        Some(l) => l.load_lecture(&content_path).map_err(|e| e.to_string()),
        None => Err("Content not loaded".to_string()),
    }
}

#[tauri::command]
pub fn load_quiz(state: State<AppState>, content_path: String) -> Result<Quiz, String> {
    let loader = state.content_loader.lock().map_err(|e| e.to_string())?;

    match &*loader {
        Some(l) => l.load_quiz(&content_path).map_err(|e| e.to_string()),
        None => Err("Content not loaded".to_string()),
    }
}
