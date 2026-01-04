use content::ContentLoader;
use glp_core::AppDatabase;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub db: AppDatabase,
    pub content_loader: Mutex<Option<ContentLoader>>,
    pub current_user_id: Mutex<Option<String>>,
}

impl AppState {
    pub fn database(&self) -> &AppDatabase {
        &self.db
    }

    pub fn get_current_user_id(&self) -> String {
        self.current_user_id
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
            .unwrap_or_else(|| "default-user".to_string())
    }

    pub fn new(content_path: PathBuf) -> Result<Self, String> {
        // Get app data directory for database
        let db_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gamified-learning-platform");

        std::fs::create_dir_all(&db_dir).map_err(|e| e.to_string())?;

        let db_path = db_dir.join("app.db");

        println!("Database path: {:?}", db_path);
        println!("Content path: {:?}", content_path);

        // Initialize database
        let db = AppDatabase::new(db_path).map_err(|e| e.to_string())?;

        // Try to load content (optional - might not exist yet)
        let content_loader = if content_path.join("manifest.json").exists() {
            match ContentLoader::new(content_path) {
                Ok(loader) => {
                    println!("Content loaded successfully");
                    Some(loader)
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load content: {}", e);
                    None
                }
            }
        } else {
            println!("No content manifest found at {:?}", content_path);
            None
        };

        Ok(Self {
            db,
            content_loader: Mutex::new(content_loader),
            current_user_id: Mutex::new(None),
        })
    }
}
