use std::path::PathBuf;
use std::fs;
use tauri::AppHandle;
use anyhow::{Result, Context};
use hf_hub::api::tokio::ApiBuilder;
use tauri::Emitter;

const DEFAULT_MODEL: &str = "unsloth/gemma-3-1b-it-GGUF";
const MODEL_FILENAME: &str = "gemma-3-1b-it-Q8_0.gguf";

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

        // Emit initial status
        app.emit("model-download-status", "Starting download...").unwrap();

        // Parse model ID to get the repo owner and name
        let model_parts = DEFAULT_MODEL.split('/').collect::<Vec<_>>();
        if model_parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid model ID format"));
        }

        // Download config.json
        let config_path = api.model(DEFAULT_MODEL.to_string()).download("config.json").await?;
        println!("Downloaded config.json to: {}", config_path.to_string_lossy());
        tokio::fs::symlink(&config_path, &parent_dir.join("config.json")).await?;

        // Download the model
        // The API takes a single string argument for the model name
        let downloaded_path = api.model(DEFAULT_MODEL.to_string()).download(MODEL_FILENAME).await?;
        println!("Downloaded model to: {}", downloaded_path.to_string_lossy());
        // Copy the file to our destination
        tokio::fs::symlink(&downloaded_path, &model_path).await?;

        // Verify the downloaded file
        if !model_path.exists() {
            return Err(anyhow::anyhow!("Model file not found after download"));
        }

        // Emit success status
        app.emit("model-download-status", "Model downloaded successfully!").unwrap();

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
