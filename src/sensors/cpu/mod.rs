use super::{Sensor, SensorError};
use crate::core::types::OS;

#[cfg(target_os = "windows")]
mod windows_cpu;

pub fn get_cpu_power_sensor() -> Result<windows_cpu::WindowsCPUSensor, SensorError> {
    Ok(windows_cpu::WindowsCPUSensor::new("Intel".to_string()))
}

enum CPUVendor {
    Intel,
    Amd,
    Other,
}

// struct CPU {
//     vendor: CPUVendor,
//     sensor: Box<dyn Sensor>,
// }

// /*
//     The CPU struct should dispatch the work to the right module depending on the OS.
//     Each module implements the Sensor trait and is accountable for backup estimation methods.
//  */


// impl CPU {
//     fn new(os: OS, vendor_id: &str) -> Result<CPU, String> {
//         let vendor_str = vendor_id.to_lowercase();
//         let vendor = if vendor_str.contains("intel") { 
//             CPUVendor::Intel
//         } else if vendor_str.contains("amd") {
//             CPUVendor::Amd
//         } else {
//             CPUVendor::Other
//         };

//         Ok(CPU {
//             vendor,
//             energy_unit: 0.0,
//         })
//     }
// }