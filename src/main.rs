#![allow(dead_code, unused_imports)]

use std::time::{self, Instant};

use database::Database;
use sensors::Sensor;
use std::time::UNIX_EPOCH;
use std::{thread, time::Duration};

mod core;
mod database;
mod sensors;

pub fn main() {
    check_permissions();
    let database = Database::new("test.db").unwrap();
    database.create_tables_if_not_exists().unwrap();
    let sensor = sensors::cpu::get_cpu_power_sensor().unwrap();
    loop {
        match sensor.read_full_data() {
            Ok(event) => {
                database.insert_cpu_data(&event).unwrap();
                println!("{}", event.data().total_power_watts);
            }
            Err(e) => {
                eprintln!("Error reading CPU sensor data: {:?}", e);
            }
        };

        thread::sleep(Duration::from_millis(1000));
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
