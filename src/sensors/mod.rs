use crate::core::types::{Event};

pub mod cpu;

pub trait Sensor {
    fn new(param: &str) -> Self;
    fn name(&self) -> &'static str;
    fn read_power_watts(&self) -> Result<Event<f64>, SensorError>;
    fn read_usage_percent(&self) -> Result<Event<f64>, SensorError>;
}

#[derive(Debug)]
pub enum SensorError {
    NotSupported,
    ReadError(String),
}