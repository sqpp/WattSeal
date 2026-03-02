#![allow(dead_code, unused_imports)]

pub mod database;
pub mod sensors;

use core::time;
use std::{
    cell::RefCell,
    rc::Rc,
    thread,
    time::{Duration, Instant, SystemTime},
};

use common::database::purge::averaging_and_purging_data;
use database::Database;
use sensors::{SensorType, create_event_from_sensors, get_hardware_info, gpu::get_gpu_list};
use sysinfo::System;

use crate::sensors::{DiskSensor, NetworkSensor, RamSensor};

pub struct CollectorApp {
    database: Database,
    sensors: Vec<SensorType>,
    iteration: u64,
    system: Rc<RefCell<System>>,
    last_update: Instant,
}

impl CollectorApp {
    pub fn new() -> Result<Self, String> {
        let database = Database::new().map_err(|e| format!("Failed to create database: {}", e))?;
        let s = System::new_all();
        Ok(CollectorApp {
            database,
            sensors: Vec::new(),
            iteration: 0,
            system: Rc::new(RefCell::new(s)),
            last_update: Instant::now(),
        })
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        let is_admin = is_admin();

        println!("\n========== INITIALIZING SYSTEM ==========\n");

        // Initialize CPU sensor
        println!("\nInitializing sensors...");
        let sensor_cpu = sensors::cpu::get_cpu_power_sensor(self.system.clone(), 0, is_admin);
        match sensor_cpu {
            Ok(sensor) => {
                println!("✓ CPU Power Sensor initialized successfully");
                self.sensors.push(sensor);
            }
            Err(e) => {
                eprintln!("✗ Failed to initialize CPU Power Sensor: {:?}", e);
            }
        }

        // Initialize GPU sensors
        let gpu_list = get_gpu_list();
        print!("\nDetected GPUs:\n");
        println!("{:#?}", gpu_list);
        for (i, gpu_name) in gpu_list.iter().enumerate() {
            print!("  GPU {}: {}\n", i, gpu_name);
            let sensor_gpu = sensors::gpu::get_gpu_power_sensor(gpu_name, i as u32);
            match sensor_gpu {
                Ok(sensor) => {
                    println!("✓ GPU Sensor {} initialized for: {}", i, gpu_name);
                    self.sensors.push(sensor);
                }
                Err(e) => {
                    println!("✗ Failed to initialize GPU sensor for {}: {:?}", gpu_name, e);
                }
            }
        }

        //Initialize RAM, Disk, Network sensors
        self.sensors.push(SensorType::RAM(RamSensor::new(self.system.clone())));
        self.sensors.push(SensorType::Disk(DiskSensor::new()));
        self.sensors.push(SensorType::Network(NetworkSensor::new()));

        // Add total power sensor
        self.sensors.push(SensorType::Total);
        // Add process sensor
        self.sensors.push(SensorType::Process);

        println!("\n========== SETTING UP DATABASE ==========");
        // Initialize database
        let mut database = Database::new().map_err(|e| format!("Failed to open database: {}", e))?;
        let table_names: Vec<&str> = self.sensors.iter().map(|s| s.table_name()).collect();

        database
            .create_tables_if_not_exists(&table_names)
            .map_err(|e| format!("Failed to create database tables: {}", e))?;
        println!("✓ Database initialized");

        println!("\n========== GATHERING HARDWARE INFORMATION ==========\n");
        self.set_hardware_info();

        Ok(())
    }

    pub fn run(&mut self) {
        println!("\n========== PURGING & AVERAGING OLD DATA ==========");
        // averaging data every hour and purge the database until the last X_hours
        averaging_and_purging_data(&mut self.database, 24, 24)
            .map_err(|e| format!("Failed averaging/purging data: {}", e))
            .ok();

        println!("\n========== POWER CONSUMPTION MONITORING ==========");
        println!("Logging data to database every second. Press Ctrl+C to stop.\n");

        loop {
            let start_time = Instant::now();
            let since_last_update_secs = self.last_update.elapsed().as_secs_f64();
            self.last_update = start_time;
            println!("\n--- Iteration {} ---", self.iteration);
            let mut total_power = 0.0;
            let event = create_event_from_sensors(&self.sensors, self.system.clone(), &mut total_power);
            let energy_wh = total_power * since_last_update_secs / 3600.0;

            match self.database.insert_event(&event) {
                Ok(_) => println!("✓ Event data saved to database"),
                Err(e) => eprintln!("✗ Failed to save event data: {:?}", e),
            }

            match self
                .database
                .update_all_time_data(energy_wh, since_last_update_secs.round() as i64)
            {
                Ok(_) => println!("✓ All-time data updated in database"),
                Err(e) => eprintln!("✗ Failed to update all-time data: {:?}", e),
            }

            for sensor_data in event.data().iter() {
                // PRINT HARDWARE DATA
                if !(sensor_data.sensor_type() == "Processes") {
                    println!("{}", sensor_data);
                }
                // PRINT PROCESS DATA
                else {
                    println!("{}", sensor_data);
                }
            }

            self.iteration += 1;
            println!(
                "All-Time Energy over {} seconds: {:.3} Wh ({} W)",
                since_last_update_secs, energy_wh, total_power
            );

            // ADJUST SLEEP DURATION TO MAINTAIN 1 SECOND INTERVALS
            let elapsed_time = start_time.elapsed();
            if elapsed_time < Duration::from_millis(1000) {
                let now = SystemTime::now();
                let time_before_next_second = now
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_millis()
                    % 1000;
                thread::sleep(Duration::from_millis(1000 - time_before_next_second as u64));
            } else {
                println!(
                    "WARNING: Iteration {} took longer than 1 second. Consider optimizing.",
                    self.iteration
                );
            }
        }
    }

    pub fn set_hardware_info(&mut self) {
        let info = get_hardware_info(&self.sensors);

        match self.database.insert_hardware_info(&info) {
            Ok(_) => println!("✓ Hardware info saved to database"),
            Err(e) => eprintln!("✗ Failed to save hardware info to database: {:?}", e),
        }

        println!("{:#?}", info);
    }
}

fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        let admin = is_admin::is_admin();
        if !admin {
            eprintln!("\u{26a0} Running without Administrator privileges. CPU power readings will use estimation.");
        }
        admin
    }

    #[cfg(target_os = "linux")]
    {
        let rapl_accessible = std::fs::read_to_string("/sys/class/powercap/intel-rapl:0/energy_uj").is_ok();
        if !rapl_accessible {
            eprintln!("\u{26a0} RAPL not accessible. CPU power readings will use estimation.");
            eprintln!("  Tip: run as root or grant read access to /sys/class/powercap/");
        }
        rapl_accessible
    }
}
