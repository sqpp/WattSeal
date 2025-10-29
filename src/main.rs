use std::{thread, time::Duration};
use sensors::Sensor;
use std::time::{UNIX_EPOCH};

mod core;
mod sensors;

pub fn main() {
    check_permissions();
    let sensor = sensors::cpu::get_cpu_power_sensor().unwrap();
    for _ in 0..5 {
        println!("Using sensor: {}", sensor.name());
        let power = sensor.read_power_watts().unwrap();
        println!("CPU Power: {:.3} W at time {:?}", power.value(), power.time().duration_since(UNIX_EPOCH).unwrap());
        let usage = sensor.read_usage_percent().unwrap();
        println!("CPU Usage: {:.2} % at time {:?}", usage.value(), usage.time().duration_since(UNIX_EPOCH).unwrap());
        thread::sleep(Duration::from_secs(1));
    }
}

fn check_permissions() {
    #[cfg(target_os = "windows")]
    {
        if !is_admin::is_admin() {
            eprintln!("This program requires Administrator privileges on Windows.");
            eprintln!("Please run this program as Administrator.");
            std::process::exit(1);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if !is_root() {
            eprintln!("This program requires root privileges on Linux.");
            eprintln!(
                "Please run with: sudo {}",
                std::env::current_exe().unwrap().display()
            );
            std::process::exit(1);
        }
    }
}