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
                eprintln!("Loading config from: {path}");
                Self::load_from_file(&path)
            } else {
                eprintln!("Config file not found: {path}");
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

        eprintln!(
            "Resolved config => api_port={:?}, api_key_set={}, power_log={}",
            config.api_port,
            config.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false),
            config.power_log.is_some()
        );

        config
    }

    fn load_from_file(path: &str) -> Self {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to read config file {path}: {e}");
                return Self::default();
            }
        };

        let content = match decode_config_text(&bytes) {
            Ok(s) => s,
            Err(msg) => {
                eprintln!("Failed to decode config file {path}: {msg}");
                return Self::default();
            }
        };

        match serde_json::from_str::<Self>(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to parse config file {path} as JSON: {e}");
                Self::default()
            }
        }
    }

    fn get_arg_value(args: &[String], arg_name: &str) -> Option<String> {
        args.iter()
            .position(|a| a == arg_name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    }
}

fn decode_config_text(bytes: &[u8]) -> Result<String, String> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(bytes[3..].to_vec()).map_err(|e| e.to_string());
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return decode_utf16(&bytes[2..], true);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return decode_utf16(&bytes[2..], false);
    }

    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return Ok(s);
    }

    // Heuristic fallback for UTF-16 without BOM, common on Windows tools.
    let zero_even = bytes.iter().step_by(2).filter(|&&b| b == 0).count();
    let zero_odd = bytes.iter().skip(1).step_by(2).filter(|&&b| b == 0).count();
    if zero_odd > bytes.len() / 8 {
        return decode_utf16(bytes, true);
    }
    if zero_even > bytes.len() / 8 {
        return decode_utf16(bytes, false);
    }

    Err("unsupported text encoding (expected UTF-8 or UTF-16)".to_string())
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> Result<String, String> {
    if !bytes.len().is_multiple_of(2) {
        return Err("UTF-16 data has odd byte length".to_string());
    }
    let mut units = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let unit = if little_endian {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        };
        units.push(unit);
    }
    String::from_utf16(&units).map_err(|e| e.to_string())
}
