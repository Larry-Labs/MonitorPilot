use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub input_names: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input_names: HashMap::new(),
        }
    }
}

pub struct ConfigManager {
    config: Mutex<AppConfig>,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let config_path = app_data_dir.join("config.json");
        let config = Self::load_from_file(&config_path).unwrap_or_else(|| {
            if config_path.exists() {
                log::warn!("配置文件 {} 解析失败，使用默认配置", config_path.display());
            }
            AppConfig::default()
        });

        Self {
            config: Mutex::new(config),
            config_path,
        }
    }

    fn load_from_file(path: &PathBuf) -> Option<AppConfig> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn get(&self) -> AppConfig {
        self.config
            .lock()
            .unwrap_or_else(|e| {
                log::error!("配置 Mutex 被污染，使用恢复值: {}", e);
                e.into_inner()
            })
            .clone()
    }

    pub fn save(&self, config: AppConfig) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("无法创建配置目录: {}", e))?;
        }

        let json =
            serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(&self.config_path, json).map_err(|e| format!("写入配置文件失败: {}", e))?;

        *self.config
            .lock()
            .unwrap_or_else(|e| {
                log::error!("配置 Mutex 被污染，使用恢复值: {}", e);
                e.into_inner()
            }) = config;
        Ok(())
    }
}
