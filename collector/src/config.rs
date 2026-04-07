use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
pub struct CollectorConfig {
    pub power_log: Option<String>,
    pub carbon_intensity: Option<String>,
    pub electricity_cost: Option<String>,
    pub api_port: Option<u16>,
    pub api_key: Option<String>,
}

impl CollectorConfig {
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().skip(1).collect();

        // 1. Check for config.json
        let config_path = args
            .iter()
            .position(|a| a == "--config")
            .and_then(|i| args.get(i + 1))
            .cloned();

        let mut config = if let Some(path) = config_path {
            if Path::new(&path).exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    serde_json::from_str(&content).unwrap_or_default()
                } else {
                    Self::default()
                }
            } else {
                Self::default()
            }
        } else {
            Self::default()
        };

        // 2. Override with CLI parameters
        if let Some(path) = Self::get_arg_value(&args, "--power-log") {
            config.power_log = Some(path);
        }
        if let Some(ci) = Self::get_arg_value(&args, "--carbon-intensity") {
            config.carbon_intensity = Some(ci);
        }
        if let Some(ec) = Self::get_arg_value(&args, "--electricity-cost") {
            config.electricity_cost = Some(ec);
        }
        if let Some(port) = Self::get_arg_value(&args, "--api-port") {
            if let Ok(p) = port.parse::<u16>() {
                config.api_port = Some(p);
            }
        }
        if let Some(key) = Self::get_arg_value(&args, "--api-key") {
            config.api_key = Some(key);
        }

        config
    }

    fn get_arg_value(args: &[String], arg_name: &str) -> Option<String> {
        args.iter()
            .position(|a| a == arg_name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    }
}
