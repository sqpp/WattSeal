use super::{Sensor, SensorError};
use crate::core::types::{CPUData, Event};
use windows_cpu::WindowsCPUSensor;

#[cfg(target_os = "windows")]
mod windows_cpu;

pub fn get_cpu_power_sensor() -> Result<impl Sensor<CPUData>, SensorError> {
    Ok(WindowsCPUSensor::new("Amd"))
}

enum CPUVendor {
    Intel,
    Amd,
    Other,
}

impl CPUVendor {
    fn from_str(vendor_str: &str) -> CPUVendor {
        let vendor_lower = vendor_str.to_lowercase();
        if vendor_lower.contains("intel") {
            CPUVendor::Intel
        } else if vendor_lower.contains("amd") {
            CPUVendor::Amd
        } else {
            CPUVendor::Other
        }
    }
}
