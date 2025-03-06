//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::constants::DEFAULT_REGISTRY;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Registry {
    pub index: String,
}

fn default_enable_package_cache() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TmanConfigFile {
    pub registry: HashMap<String, Registry>,

    pub admin_token: Option<String>,
    pub user_token: Option<String>,

    #[serde(default = "default_enable_package_cache")]
    pub enable_package_cache: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TmanConfig {
    pub registry: HashMap<String, Registry>,

    pub config_file: Option<String>,
    pub admin_token: Option<String>,
    pub user_token: Option<String>,

    pub verbose: bool,
    pub assume_yes: bool,

    pub enable_package_cache: bool,
}

impl Default for TmanConfig {
    fn default() -> Self {
        let mut registry = HashMap::new();

        registry.entry("default".to_string()).or_insert(Registry {
            index: DEFAULT_REGISTRY.to_string(),
        });

        Self {
            registry,
            config_file: Some(
                get_default_config_path().to_string_lossy().to_string(),
            ),
            admin_token: None,
            user_token: None,
            verbose: false,
            assume_yes: false,
            enable_package_cache: true,
        }
    }
}

// Determine the tman home directory based on the platform.
fn get_default_home_dir() -> PathBuf {
    let mut home_dir =
        dirs::home_dir().expect("Cannot determine home directory.");
    if cfg!(target_os = "windows") {
        home_dir.push("AppData");
        home_dir.push("Roaming");
        home_dir.push("tman");
    } else {
        home_dir.push(".tman");
    }
    home_dir
}

/// Get the default configuration file path, located under the default home
/// directory as `config.json`.
fn get_default_config_path() -> PathBuf {
    let mut config_path = get_default_home_dir();
    config_path.push("config.json");
    config_path
}

/// Get the default package cache folder, located under the default home
/// directory as `package_cache/`.
pub fn get_default_package_cache_folder() -> PathBuf {
    let mut cache_path = get_default_home_dir();
    cache_path.push("package_cache");
    cache_path
}

// Read the configuration from the specified path.
pub fn read_config(
    config_file_path: &Option<String>,
) -> Result<Option<TmanConfigFile>> {
    let config_path = match config_file_path {
        Some(path) => PathBuf::from(path),
        None => get_default_config_path(),
    };

    let config_data = fs::read_to_string(config_path.clone());

    if config_path.exists() {
        match config_data {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(config) => Ok(Some(config)),
                Err(e) => {
                    Err(anyhow::anyhow!("Failed to parse config file: {}", e))
                }
            },
            Err(e) => Err(anyhow::anyhow!("Failed to read config file: {}", e)),
        }
    } else {
        Ok(None)
    }
}
