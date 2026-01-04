use content::ContentLoader;
use glp_core::AppDatabase;
use glp_core::db::repos::CurriculumRepository;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub db: AppDatabase,
    pub content_loader: Mutex<Option<ContentLoader>>,
    pub current_user_id: Mutex<Option<String>>,
    pub app_data_dir: PathBuf,
    pub active_curriculum_id: Mutex<Option<String>>,
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

    pub fn get_active_curriculum_id(&self) -> Option<String> {
        self.active_curriculum_id
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }

    pub fn app_data_dir(&self) -> &PathBuf {
        &self.app_data_dir
    }

    pub fn new(_content_path: PathBuf) -> Result<Self, String> {
        // Get app data directory for database and curricula
        let app_data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gamified-learning-platform");

        std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(app_data_dir.join("curricula")).map_err(|e| e.to_string())?;

        let db_path = app_data_dir.join("app.db");

        println!("Database path: {:?}", db_path);
        println!("App data dir: {:?}", app_data_dir);

        // Initialize database
        let db = AppDatabase::new(db_path).map_err(|e| e.to_string())?;

        // Try to load the active curriculum from database
        let (content_loader, active_curriculum_id) = db
            .with_connection(|conn| {
                match CurriculumRepository::get_active(conn)? {
                    Some(curriculum) => {
                        let content_path = app_data_dir.join(&curriculum.content_path);
                        if content_path.join("manifest.json").exists() {
                            match ContentLoader::new(content_path) {
                                Ok(loader) => {
                                    println!("Loaded active curriculum: {}", curriculum.name);
                                    Ok((Some(loader), Some(curriculum.id)))
                                }
                                Err(e) => {
                                    eprintln!("Warning: Failed to load curriculum content: {}", e);
                                    Ok((None, None))
                                }
                            }
                        } else {
                            eprintln!("Warning: Active curriculum content not found at {:?}", content_path);
                            Ok((None, None))
                        }
                    }
                    None => {
                        println!("No active curriculum set");
                        Ok((None, None))
                    }
                }
            })
            .map_err(|e| e.to_string())?;

        Ok(Self {
            db,
            content_loader: Mutex::new(content_loader),
            current_user_id: Mutex::new(None),
            app_data_dir,
            active_curriculum_id: Mutex::new(active_curriculum_id),
        })
    }

    /// Load a curriculum by ID and set it as active
    pub fn load_curriculum(&self, curriculum_id: &str) -> Result<(), String> {
        let curriculum = self.db
            .with_connection(|conn| {
                CurriculumRepository::get(conn, curriculum_id)
            })
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Curriculum not found: {}", curriculum_id))?;

        let content_path = self.app_data_dir.join(&curriculum.content_path);
        let loader = ContentLoader::new(content_path).map_err(|e| e.to_string())?;

        // Update content loader
        let mut content_guard = self.content_loader.lock().map_err(|e| e.to_string())?;
        *content_guard = Some(loader);

        // Update active curriculum ID
        let mut id_guard = self.active_curriculum_id.lock().map_err(|e| e.to_string())?;
        *id_guard = Some(curriculum_id.to_string());

        // Update database
        self.db
            .with_connection(|conn| {
                CurriculumRepository::set_active(conn, curriculum_id)
            })
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Unload the current curriculum
    pub fn unload_curriculum(&self) -> Result<(), String> {
        let mut content_guard = self.content_loader.lock().map_err(|e| e.to_string())?;
        *content_guard = None;

        let mut id_guard = self.active_curriculum_id.lock().map_err(|e| e.to_string())?;
        *id_guard = None;

        Ok(())
    }
}
