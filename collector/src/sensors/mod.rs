use std::time::SystemTime;

use crate::database::{Event, SensorData, TotalData};

pub mod cpu;
pub mod gpu;

pub use cpu::CPUSensor;
pub use gpu::GPUSensor;

pub enum SensorType {
    CPU(CPUSensor),
    GPU(GPUSensor),
    Total,
}

impl Sensor for SensorType {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_full_data(),
            SensorType::GPU(sensor) => sensor.read_full_data(),
            SensorType::Total => Err(SensorError::NotSupported),
        }
    }
}

pub trait Sensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError>;
}

#[derive(Debug)]
pub enum SensorError {
    NotSupported,
    ReadError(String),
}

pub fn create_event_from_sensors(sensors: &Vec<SensorType>) -> Event {
    let time = SystemTime::now();
    let mut data = Vec::new();
    let mut total_power = 0.0;
    for sensor in sensors {
        if let SensorType::Total = sensor {
            continue;
        }
        let sensor_data = sensor.read_full_data();
        match sensor_data {
            Ok(d) => {
                println!("{}", d);
                if let Some(power) = d.total_power_watts() {
                    total_power += power;
                }
                data.push(d);
            }
            Err(e) => eprintln!("✗ Error reading sensor data: {:?}", e),
        }
    }
    data.push(SensorData::Total(TotalData {
        total_power_watts: total_power,
    }));
    Event::new(time, data)
}
