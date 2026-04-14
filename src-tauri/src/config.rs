use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub input_names: HashMap<String, String>,
    #[serde(default)]
    pub monitor_order: Vec<String>,
    #[serde(default)]
    pub presets: Vec<InputPreset>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputPreset {
    pub name: String,
    pub inputs: HashMap<String, u16>,
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
            } else {
                log::debug!("配置文件不存在，使用默认配置: {}", config_path.display());
            }
            AppConfig::default()
        });

        log::info!(
            "配置已加载: {} 个自定义名称 | 路径: {}",
            config.input_names.len(),
            config_path.display()
        );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_config_default_is_empty() {
        let config = AppConfig::default();
        assert!(config.input_names.is_empty());
        assert!(config.monitor_order.is_empty());
        assert!(config.presets.is_empty());
    }

    #[test]
    fn app_config_serialization_roundtrip() {
        let mut config = AppConfig::default();
        config.input_names.insert("1-15".to_string(), "MacBook".to_string());
        config.input_names.insert("1-17".to_string(), "Ubuntu".to_string());

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.input_names.len(), 2);
        assert_eq!(deserialized.input_names["1-15"], "MacBook");
        assert_eq!(deserialized.input_names["1-17"], "Ubuntu");
    }

    #[test]
    fn app_config_deserialize_missing_fields_uses_default() {
        let json = "{}";
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert!(config.input_names.is_empty());
        assert!(config.monitor_order.is_empty());
        assert!(config.presets.is_empty());
    }

    #[test]
    fn app_config_backward_compatible_with_v1_format() {
        let json = r#"{"input_names":{"1-15":"MacBook"}}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.input_names["1-15"], "MacBook");
        assert!(config.presets.is_empty());
        assert!(config.monitor_order.is_empty());
    }

    #[test]
    fn preset_serialization_roundtrip() {
        let preset = InputPreset {
            name: "工作模式".to_string(),
            inputs: [("1".to_string(), 15u16), ("2".to_string(), 17u16)]
                .into_iter()
                .collect(),
        };
        let json = serde_json::to_string(&preset).unwrap();
        let deserialized: InputPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "工作模式");
        assert_eq!(deserialized.inputs["1"], 15);
        assert_eq!(deserialized.inputs["2"], 17);
    }

    #[test]
    fn config_with_presets_roundtrip() {
        let mut config = AppConfig::default();
        config.presets.push(InputPreset {
            name: "办公".to_string(),
            inputs: [("1".to_string(), 15u16)].into_iter().collect(),
        });
        config.monitor_order = vec!["LG ULTRAGEAR".to_string()];

        let json = serde_json::to_string(&config).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.presets.len(), 1);
        assert_eq!(loaded.presets[0].name, "办公");
        assert_eq!(loaded.monitor_order, vec!["LG ULTRAGEAR"]);
    }

    #[test]
    fn config_manager_new_with_nonexistent_path() {
        let dir = std::env::temp_dir().join("monitorpilot_test_nonexistent");
        let _ = fs::remove_dir_all(&dir);
        let manager = ConfigManager::new(dir.clone());
        assert!(manager.get().input_names.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_manager_save_and_get() {
        let dir = std::env::temp_dir().join("monitorpilot_test_save");
        let _ = fs::remove_dir_all(&dir);

        let manager = ConfigManager::new(dir.clone());
        let mut config = AppConfig::default();
        config.input_names.insert("1-15".to_string(), "MacBook".to_string());

        manager.save(config).unwrap();
        let loaded = manager.get();
        assert_eq!(loaded.input_names["1-15"], "MacBook");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_manager_save_creates_directory() {
        let dir = std::env::temp_dir().join("monitorpilot_test_mkdir/deep/path");
        let _ = fs::remove_dir_all(std::env::temp_dir().join("monitorpilot_test_mkdir"));

        let manager = ConfigManager::new(dir.clone());
        let config = AppConfig::default();
        assert!(manager.save(config).is_ok());
        assert!(dir.join("config.json").exists());

        let _ = fs::remove_dir_all(std::env::temp_dir().join("monitorpilot_test_mkdir"));
    }

    #[test]
    fn config_manager_load_corrupted_file_uses_default() {
        let dir = std::env::temp_dir().join("monitorpilot_test_corrupt");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("config.json"), "not valid json!!!").unwrap();

        let manager = ConfigManager::new(dir.clone());
        assert!(manager.get().input_names.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_manager_persistence_across_instances() {
        let dir = std::env::temp_dir().join("monitorpilot_test_persist");
        let _ = fs::remove_dir_all(&dir);

        let manager1 = ConfigManager::new(dir.clone());
        let mut config = AppConfig::default();
        config.input_names.insert("2-18".to_string(), "Ubuntu".to_string());
        manager1.save(config).unwrap();
        drop(manager1);

        let manager2 = ConfigManager::new(dir.clone());
        assert_eq!(manager2.get().input_names["2-18"], "Ubuntu");

        let _ = fs::remove_dir_all(&dir);
    }
}
