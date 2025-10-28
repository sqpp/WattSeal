pub mod cpu;

pub trait Sensor {
    fn new<T>(param: T) -> Self;
    fn name(&self) -> &'static str;
    fn read_power_watts(&self) -> Result<f64, SensorError>;
    fn read_usage_percent(&self) -> Result<f64, SensorError>;
}

#[derive(Debug)]
pub enum SensorError {
    NotSupported,
    ReadError(String),
}