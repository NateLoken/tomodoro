use std::{env, fs, io::ErrorKind};

use color_eyre::eyre::Result;
use ratatui::style::Color;

use serde::Deserialize;

use crate::timer::{PhaseSpec, TimeUnit};

#[derive(Debug, Clone, Deserialize)]
pub struct PhaseConfig {
    pub phases: Vec<PhasePreset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhasePreset {
    pub name: String,
    pub duration: f64,
    pub unit: TimeUnit,
    pub color: Color,
}

impl PhaseConfig {
    pub fn new() -> Result<(Self, Option<String>)> {
        let mut config_path = env::current_dir()?;
        config_path.push("config.toml");

        let config_str = match fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                let default_cfg = r#"[[ phases ]]
name = "Focus"
duration = 25.0
unit = "Minutes"
color = "Red"

[[ phases ]]
name = "Rest"
duration = 5.0
unit = "Minutes"
color = "Blue"
"#;
                fs::write(&config_path, default_cfg)?;
                default_cfg.to_string()
            }
            Err(err) => return Err(err.into()),
        };

        let (config, warning): (Self, Option<String>) = match toml::from_str(&config_str) {
            Ok(cfg) => (cfg, None),
            Err(parse_error) => {
                let warning = format!(
                    "warning: failed to parse {}.\n{}\nusing built-in default phases.",
                    config_path.display(),
                    parse_error
                );
                (PhaseConfig::default(), Some(warning))
            }
        };

        Ok((config, warning))
    }

    fn default() -> Self {
        Self {
            phases: vec![
                PhasePreset {
                    name: "Focus".to_string(),
                    duration: 25.0,
                    unit: TimeUnit::Minutes,
                    color: Color::Red,
                },
                PhasePreset {
                    name: "Rest".to_string(),
                    duration: 5.0,
                    unit: TimeUnit::Minutes,
                    color: Color::Blue,
                },
            ],
        }
    }
}

impl PhasePreset {
    pub fn total_seconds(&self) -> f64 {
        self.unit.to_seconds(self.duration)
    }

    pub fn to_spec(&self) -> PhaseSpec {
        PhaseSpec::new(self.name.clone(), self.duration, self.unit)
    }
}
