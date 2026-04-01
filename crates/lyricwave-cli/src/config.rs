use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProfileConfig {
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub fps: Option<u32>,
    pub source_lang: Option<String>,
    pub target_lang: Option<String>,
    pub asr_provider: Option<String>,
    pub translator_provider: Option<String>,
    pub vibevoice_dir: Option<PathBuf>,
    pub model_path: Option<String>,
    pub python_bin: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct FileConfig {
    #[serde(default)]
    default: ProfileConfig,
    #[serde(default)]
    profiles: BTreeMap<String, ProfileConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedConfig {
    pub profile_name: String,
    pub values: ProfileConfig,
    pub source_path: Option<PathBuf>,
}

impl ResolvedConfig {
    pub fn load(config_path: Option<&Path>, profile: &str) -> Result<Self> {
        let path = match config_path {
            Some(path) => Some(path.to_path_buf()),
            None => default_config_path(),
        };

        let Some(path) = path else {
            return Ok(Self {
                profile_name: profile.to_string(),
                values: ProfileConfig::default(),
                source_path: None,
            });
        };

        if !path.exists() {
            return Ok(Self {
                profile_name: profile.to_string(),
                values: ProfileConfig::default(),
                source_path: Some(path),
            });
        }

        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        let parsed: FileConfig = toml::from_str(&raw)
            .with_context(|| format!("invalid TOML config {}", path.display()))?;

        let profile_cfg = parsed.profiles.get(profile).cloned().unwrap_or_default();
        let values = merge_profile(&parsed.default, &profile_cfg);

        Ok(Self {
            profile_name: profile.to_string(),
            values,
            source_path: Some(path),
        })
    }
}

fn merge_profile(defaults: &ProfileConfig, selected: &ProfileConfig) -> ProfileConfig {
    ProfileConfig {
        sample_rate: selected.sample_rate.or(defaults.sample_rate),
        channels: selected.channels.or(defaults.channels),
        fps: selected.fps.or(defaults.fps),
        source_lang: selected
            .source_lang
            .clone()
            .or_else(|| defaults.source_lang.clone()),
        target_lang: selected
            .target_lang
            .clone()
            .or_else(|| defaults.target_lang.clone()),
        asr_provider: selected
            .asr_provider
            .clone()
            .or_else(|| defaults.asr_provider.clone()),
        translator_provider: selected
            .translator_provider
            .clone()
            .or_else(|| defaults.translator_provider.clone()),
        vibevoice_dir: selected
            .vibevoice_dir
            .clone()
            .or_else(|| defaults.vibevoice_dir.clone()),
        model_path: selected
            .model_path
            .clone()
            .or_else(|| defaults.model_path.clone()),
        python_bin: selected
            .python_bin
            .clone()
            .or_else(|| defaults.python_bin.clone()),
    }
}

fn default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("lyricwave").join("config.toml"))
}
