use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an imported curriculum/content pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curriculum {
    /// Unique identifier (UUID)
    pub id: String,
    /// Display name of the curriculum
    pub name: String,
    /// Version string (e.g., "1.0.0")
    pub version: String,
    /// Optional description
    pub description: Option<String>,
    /// Optional author name
    pub author: Option<String>,
    /// When the curriculum was imported
    pub imported_at: DateTime<Utc>,
    /// Path to the content directory (relative to app data)
    pub content_path: String,
    /// Whether this curriculum is currently active
    pub is_active: bool,
}

impl Curriculum {
    pub fn new(
        name: String,
        version: String,
        content_path: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            version,
            description: None,
            author: None,
            imported_at: Utc::now(),
            content_path,
            is_active: false,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }
}

/// Summary info about a curriculum (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub imported_at: DateTime<Utc>,
    pub is_active: bool,
}

impl From<&Curriculum> for CurriculumSummary {
    fn from(c: &Curriculum) -> Self {
        Self {
            id: c.id.clone(),
            name: c.name.clone(),
            version: c.version.clone(),
            description: c.description.clone(),
            author: c.author.clone(),
            imported_at: c.imported_at,
            is_active: c.is_active,
        }
    }
}
