use std::path::PathBuf;
use std::fs;
use tauri::AppHandle;
use anyhow::{Result, Context};
use hf_hub::api::tokio::{ApiBuilder, Progress};
use tauri::Emitter;

const DEFAULT_MODEL: &str = "unsloth/gemma-3-1b-it-GGUF";
const MODEL_FILENAME: &str = "gemma-3-1b-it-Q8_0.gguf";

// Custom progress tracker that emits download progress
#[derive(Clone)]
struct DownloadProgress {
    app: AppHandle,
    total: usize,
    file_name: String,
    tracked_bytes: std::sync::Arc<std::sync::atomic::AtomicUsize>, // Track bytes across chunks
}

impl Progress for DownloadProgress {
    async fn init(&mut self, size: usize, filename: &str) {
        self.total = size;
        self.file_name = filename.to_string();
        self.tracked_bytes.store(0, std::sync::atomic::Ordering::SeqCst);
        
        // Emit single event with all information
        if let Err(e) = self.app.emit("model-download-progress", serde_json::json!({
            "status": "initializing",
            "message": format!("Starting download of {} ({} bytes)", filename, size),
            "filename": filename,
            "current": 0,
            "total": size,
            "percentage": 0.0,
            "formatted": "0.0%"
        })) {
            println!("Failed to emit initialization progress: {}", e);
        }
    }

    async fn update(&mut self, size: usize) {
        // Track bytes cumulatively across chunked downloads
        let previous = self.tracked_bytes.fetch_add(size, std::sync::atomic::Ordering::SeqCst);
        let cumulative_bytes = previous + size;
        
        let progress = if self.total > 0 {
            (cumulative_bytes as f32 / self.total as f32) * 100.0
        } else {
            0.0
        };

        // println!("Raw Progress: {}, size: {}, current: {}, cumulative: {}, total: {}", 
        //     progress, size, self.current, cumulative_bytes, self.total);
        
        let formatted = format!("{:.1}%", progress);
        
        // Emit single event with all progress information using the cumulative bytes
        if let Err(e) = self.app.emit("model-download-progress", serde_json::json!({
            "status": "downloading",
            "message": format!("Downloading {} - {}", self.file_name, formatted),
            "filename": self.file_name,
            "current": cumulative_bytes,
            "total": self.total,
            "percentage": progress,
            "formatted": formatted
        })) {
            println!("Failed to emit download progress: {}", e);
        }
    }

    async fn finish(&mut self) {
        let formatted = "100.0%";
        
        // Emit single completion event
        if let Err(e) = self.app.emit("model-download-progress", serde_json::json!({
            "status": "completed",
            "message": format!("Finished downloading {}", self.file_name),
            "filename": self.file_name,
            "current": self.total,
            "total": self.total,
            "percentage": 100.0,
            "formatted": formatted
        })) {
            println!("Failed to emit completion progress: {}", e);
        }
    }
}

pub struct ModelManager {
    model_dir: PathBuf,
}

impl ModelManager {
    pub fn new() -> Result<Self> {
        let model_dir = Self::get_model_dir()?;
        Ok(Self { model_dir })
    }

    fn get_model_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let model_dir = if cfg!(windows) {
            home_dir.join("AppData").join("Roaming").join("gui").join("models")
        } else {
            home_dir.join(".cache").join("gui").join("models")
        };

        // Create the directory if it doesn't exist
        fs::create_dir_all(&model_dir)?;
        Ok(model_dir)
    }

    pub fn get_model_path(&self) -> PathBuf {
        self.model_dir.join(DEFAULT_MODEL).join(MODEL_FILENAME)
    }

    pub async fn check_model_exists(&self) -> bool {
        self.get_model_path().exists()
    }

    pub async fn download_model(&self, app: &AppHandle) -> Result<()> {
        let model_path = self.get_model_path();
        let parent_dir = model_path.parent().unwrap();

        println!("Model dir: {}", self.model_dir.to_string_lossy());

        // Create parent directory if it doesn't exist
        tokio::fs::create_dir_all(parent_dir).await?;

        // Initialize Hugging Face Hub
        let cache_dir = self.model_dir.clone();
        let api = ApiBuilder::new().with_cache_dir(cache_dir).build()?;

        // Parse model ID to get the repo owner and name
        let model_parts = DEFAULT_MODEL.split('/').collect::<Vec<_>>();
        if model_parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid model ID format"));
        }

        // Initialize progress handler for config file
        let progress_handler = DownloadProgress {
            app: app.clone(),
            total: 0,
            file_name: String::new(),
            tracked_bytes: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        };
        
        let config_path = api.model(DEFAULT_MODEL.to_string())
            .download_with_progress("config.json", progress_handler.clone())
            .await?;
            
        println!("Downloaded config.json to: {}", config_path.to_string_lossy());
        
        // Create symbolic link to config.json
        if let Err(e) = tokio::fs::symlink(&config_path, &parent_dir.join("config.json")).await {
            println!("Error creating symlink for config.json: {}", e);
            // Fallback to copy if symlink fails
            tokio::fs::copy(&config_path, &parent_dir.join("config.json")).await?;
        }
        
        let downloaded_path = api.model(DEFAULT_MODEL.to_string())
            .download_with_progress(MODEL_FILENAME, progress_handler)
            .await?;
            
        println!("Downloaded model to: {}", downloaded_path.to_string_lossy());
        
        // Create symbolic link to model file
        if let Err(e) = tokio::fs::symlink(&downloaded_path, &model_path).await {
            println!("Error creating symlink for model: {}", e);
            // Fallback to copy if symlink fails
            tokio::fs::copy(&downloaded_path, &model_path).await?;
        }

        // Verify the downloaded file
        if !model_path.exists() {
            return Err(anyhow::anyhow!("Model file not found after download"));
        }

        Ok(())
    }
}

// Internal function for use in the setup method
pub async fn check_and_download_model(app: &AppHandle) -> Result<(), String> {
    let model_manager = ModelManager::new().map_err(|e| e.to_string())?;
    
    if !model_manager.check_model_exists().await {
        model_manager.download_model(app)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
