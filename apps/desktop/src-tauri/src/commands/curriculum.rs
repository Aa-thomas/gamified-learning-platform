use crate::state::AppState;
use content::{import_content_pack, validate_content_pack, get_content_stats, ContentStats};
use glp_core::db::repos::CurriculumRepository;
use glp_core::models::Curriculum;
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

#[derive(Serialize)]
pub struct CurriculumInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub imported_at: String,
    pub is_active: bool,
    pub stats: Option<ContentStats>,
}

impl From<Curriculum> for CurriculumInfo {
    fn from(c: Curriculum) -> Self {
        Self {
            id: c.id,
            name: c.name,
            version: c.version,
            description: c.description,
            author: c.author,
            imported_at: c.imported_at.to_rfc3339(),
            is_active: c.is_active,
            stats: None,
        }
    }
}

#[derive(Serialize)]
pub struct ValidationResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub stats: Option<ContentStats>,
}

#[derive(Serialize)]
pub struct ImportResponse {
    pub success: bool,
    pub curriculum_id: Option<String>,
    pub error: Option<String>,
}

/// Validate a content pack without importing it
#[tauri::command]
pub fn validate_curriculum(source_path: String) -> Result<ValidationResponse, String> {
    let path = PathBuf::from(&source_path);
    let result = validate_content_pack(&path).map_err(|e| e.to_string())?;
    
    let (name, version, description, author, stats) = if let Some(ref manifest) = result.manifest {
        (
            Some(manifest.title.clone()),
            Some(manifest.version.clone()),
            Some(manifest.description.clone()),
            Some(manifest.author.clone()),
            Some(get_content_stats(manifest)),
        )
    } else {
        (None, None, None, None, None)
    };

    Ok(ValidationResponse {
        is_valid: result.is_valid,
        errors: result.errors,
        warnings: result.warnings,
        name,
        version,
        description,
        author,
        stats,
    })
}

/// Import a curriculum from a folder path
#[tauri::command]
pub fn import_curriculum(
    state: State<AppState>,
    source_path: String,
    set_active: bool,
) -> Result<ImportResponse, String> {
    let source = PathBuf::from(&source_path);
    
    // First validate
    let validation = validate_content_pack(&source).map_err(|e| e.to_string())?;
    if !validation.is_valid {
        return Ok(ImportResponse {
            success: false,
            curriculum_id: None,
            error: Some(validation.errors.join("; ")),
        });
    }

    let manifest = validation.manifest.ok_or("No manifest found")?;
    
    // Check if already exists
    let exists = state.db
        .with_connection(|conn| {
            CurriculumRepository::exists_by_name_version(conn, &manifest.title, &manifest.version)
        })
        .map_err(|e| e.to_string())?;

    if exists {
        return Ok(ImportResponse {
            success: false,
            curriculum_id: None,
            error: Some(format!(
                "Curriculum '{}' version '{}' already exists",
                manifest.title, manifest.version
            )),
        });
    }

    // Create curriculum record
    let curriculum = Curriculum::new(
        manifest.title.clone(),
        manifest.version.clone(),
        format!("curricula/{}", uuid::Uuid::new_v4()),
    )
    .with_description(manifest.description.clone())
    .with_author(manifest.author.clone());

    // Import content files
    let content_path = import_content_pack(
        &source,
        state.app_data_dir(),
        &curriculum.id,
    ).map_err(|e| e.to_string())?;

    // Update curriculum with actual content path
    let mut curriculum = curriculum;
    curriculum.content_path = content_path.to_string_lossy().to_string();

    // Save to database
    state.db
        .with_connection(|conn| {
            CurriculumRepository::create(conn, &curriculum)
        })
        .map_err(|e| e.to_string())?;

    let curriculum_id = curriculum.id.clone();

    // Optionally set as active
    if set_active {
        state.load_curriculum(&curriculum_id)?;
    }

    Ok(ImportResponse {
        success: true,
        curriculum_id: Some(curriculum_id),
        error: None,
    })
}

/// List all imported curricula
#[tauri::command]
pub fn list_curricula(state: State<AppState>) -> Result<Vec<CurriculumInfo>, String> {
    let curricula = state.db
        .with_connection(|conn| {
            CurriculumRepository::get_all(conn)
        })
        .map_err(|e| e.to_string())?;

    Ok(curricula.into_iter().map(CurriculumInfo::from).collect())
}

/// Get the currently active curriculum
#[tauri::command]
pub fn get_active_curriculum(state: State<AppState>) -> Result<Option<CurriculumInfo>, String> {
    let curriculum = state.db
        .with_connection(|conn| {
            CurriculumRepository::get_active(conn)
        })
        .map_err(|e| e.to_string())?;

    Ok(curriculum.map(CurriculumInfo::from))
}

/// Switch to a different curriculum
#[tauri::command]
pub fn switch_curriculum(state: State<AppState>, curriculum_id: String) -> Result<(), String> {
    state.load_curriculum(&curriculum_id)
}

/// Delete a curriculum
#[tauri::command]
pub fn delete_curriculum(
    state: State<AppState>,
    curriculum_id: String,
    delete_progress: bool,
) -> Result<(), String> {
    // Check if this is the active curriculum
    let active_id = state.get_active_curriculum_id();
    if active_id.as_ref() == Some(&curriculum_id) {
        state.unload_curriculum()?;
    }

    // Delete from database (and optionally progress)
    state.db
        .with_connection(|conn| {
            if delete_progress {
                CurriculumRepository::delete_with_progress(conn, &curriculum_id)
            } else {
                CurriculumRepository::delete(conn, &curriculum_id)
            }
        })
        .map_err(|e| e.to_string())?;

    // Delete content files
    content::delete_content_pack(state.app_data_dir(), &curriculum_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get a specific curriculum by ID
#[tauri::command]
pub fn get_curriculum(state: State<AppState>, curriculum_id: String) -> Result<Option<CurriculumInfo>, String> {
    let curriculum = state.db
        .with_connection(|conn| {
            CurriculumRepository::get(conn, &curriculum_id)
        })
        .map_err(|e| e.to_string())?;

    Ok(curriculum.map(CurriculumInfo::from))
}
