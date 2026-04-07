pub mod database;
pub mod sensors;
pub mod config;
pub mod api;

use std::{
    cell::RefCell,
    rc::Rc,
    thread,
    time::{Duration, Instant, SystemTime},
};

#[cfg(not(debug_assertions))]
use common::logging::start_log_session;
use common::{clog, database::purge::averaging_and_purging_data};
use database::Database;
use sensors::{SensorType, create_event_from_sensors, get_hardware_info, gpu::get_gpu_list};
use sysinfo::System;

use crate::sensors::{DiskSensor, NetworkSensor, RamSensor};

/// Background sensor-collection application.
pub struct CollectorApp {
    database: Database,
    sensors: Vec<SensorType>,
    system: Rc<RefCell<System>>,
    last_update: Instant,
    last_purge: Instant,
    #[cfg(debug_assertions)]
    iteration: u64,
    power_log_path: Option<String>,
    pending_config: Option<config::CollectorConfig>,
}

impl CollectorApp {
    /// Creates a new collector with a database connection.
    pub fn new() -> Result<Self, String> {
        let database = Database::new().map_err(|e| format!("Failed to create database: {e}"))?;
        let s = System::new_all();
        Ok(CollectorApp {
            database,
            sensors: Vec::new(),
            system: Rc::new(RefCell::new(s)),
            last_update: Instant::now(),
            last_purge: Instant::now(),
            #[cfg(debug_assertions)]
            iteration: 0,
            power_log_path: None,
            pending_config: None,
        })
    }

    /// Set configuration loaded from args/file
    pub fn with_config(mut self, config: config::CollectorConfig) -> Self {
        self.power_log_path = config.power_log.clone();
        self.pending_config = Some(config);
        self
    }


    fn purge_and_average(&mut self) {
        thread::spawn(|| {
            if let Ok(mut db) = Database::new() {
                let _ = averaging_and_purging_data(&mut db, 24, 24);
            }
        });
        self.last_purge = Instant::now();
    }

    /// Detects hardware sensors, creates database tables, and saves hardware info.
    pub fn initialize(&mut self) -> Result<(), String> {
        let is_admin = is_admin();

        #[cfg(not(debug_assertions))]
        start_log_session();

        clog!("\n========== INITIALIZING SYSTEM ==========\n");

        // CPU sensor
        clog!("Initializing sensors...");
        match sensors::cpu::get_cpu_power_sensor(self.system.clone(), 0, is_admin) {
            Ok(sensor) => {
                clog!("✓ CPU Power Sensor initialized successfully");
                self.sensors.push(sensor);
            }
            Err(e) => clog!("✗ Failed to initialize CPU Power Sensor: {:?}", e),
        }

        // GPU sensors
        let gpu_list = get_gpu_list();
        clog!("\nDetected GPUs: {gpu_list:#?}");
        for (i, gpu_name) in gpu_list.iter().enumerate() {
            match sensors::gpu::get_gpu_power_sensor(gpu_name, i as u32) {
                Ok(sensor) => {
                    clog!("✓ GPU Sensor {i} initialized: {gpu_name}");
                    self.sensors.push(sensor);
                }
                Err(e) => clog!("✗ Failed to initialize GPU sensor for {gpu_name}: {:?}", e),
            }
        }

        // RAM, Disk, Network sensors
        self.sensors.push(SensorType::RAM(RamSensor::new(self.system.clone())));
        self.sensors.push(SensorType::Disk(DiskSensor::new()));
        self.sensors.push(SensorType::Network(NetworkSensor::new()));
        self.sensors.push(SensorType::Total);
        self.sensors.push(SensorType::Process);

        // Database
        clog!("\n========== SETTING UP DATABASE ==========");
        let table_names: Vec<&str> = self.sensors.iter().map(|s| s.table_name()).collect();
        self.database
            .create_tables_if_not_exists(&table_names)
            .map_err(|e| format!("Failed to create database tables: {e}"))?;
        clog!("✓ Database initialized");

        // Hardware info
        clog!("\n========== GATHERING HARDWARE INFORMATION ==========\n");
        let info = get_hardware_info(&self.sensors);
        match self.database.insert_hardware_info(&info) {
            Ok(_) => clog!("✓ Hardware info saved"),
            Err(e) => clog!("✗ Failed to save hardware info: {e}"),
        }

        // Apply config
        if let Some(config) = self.pending_config.take() {
            let mut ui_settings = self.database.load_ui_settings().unwrap_or_default().unwrap_or(common::database::UiSettings {
                language: "EN".to_string(),
                carbon_intensity: "World average".to_string(),
                kwh_cost: "World average".to_string(),
                theme: "Hunting".to_string(),
            });
            let mut changed = false;
            if let Some(ci) = config.carbon_intensity {
                ui_settings.carbon_intensity = ci;
                changed = true;
            }
            if let Some(ec) = config.electricity_cost {
                ui_settings.kwh_cost = ec;
                changed = true;
            }
            if changed {
                let _ = self.database.save_ui_settings(&ui_settings);
                clog!("✓ CLI/Config UI settings applied");
            }
            
            if let Some(port) = config.api_port {
                api::start_api_server(port, config.api_key);
            }
        }

        clog!("Initialization complete");
        Ok(())
    }

    /// Runs the collection loop, sampling sensors every second.
    pub fn run(&mut self) {
        // Purge/averaging runs in a separate thread so collection starts immediately.
        self.purge_and_average();

        #[cfg(debug_assertions)]
        println!(
            "\n========== POWER CONSUMPTION MONITORING ==========\nLogging to database every second. Press Ctrl+C to stop.\n"
        );

        loop {
            if self.last_purge.elapsed() > Duration::from_secs(24 * 3600) {
                self.purge_and_average();
            }
            let start_time = Instant::now();
            let since_last_update_secs = self.last_update.elapsed().as_secs_f64();
            self.last_update = start_time;

            #[cfg(debug_assertions)]
            println!("\n--- Iteration {} ---", self.iteration);

            let event = create_event_from_sensors(&self.sensors, self.system.clone());

            #[cfg(debug_assertions)]
            {
                let start = Instant::now();
                let result = self
                    .database
                    .insert_event_and_update_energy(&event, since_last_update_secs);
                let duration = start.elapsed();
                match result {
                    Ok(_) => println!("✓ Event data saved to database (took {:.2?})", duration),
                    Err(e) => eprintln!("✗ Failed to save event data: {:?}", e),
                }
            }

            #[cfg(not(debug_assertions))]
            let _ = self
                .database
                .insert_event_and_update_energy(&event, since_last_update_secs);

            if let Some(ref path) = self.power_log_path {
                use std::io::Write;
                let file_exists = std::path::Path::new(path).exists();
                let is_empty = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) == 0;
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
                    if !file_exists || is_empty {
                        let _ = writeln!(f, "{}", common::types::Event::to_csv_header());
                    }
                    let _ = writeln!(f, "{}", event.to_csv_row());
                }
            }

            #[cfg(debug_assertions)]
            for sensor_data in event.data() {
                println!("{sensor_data}");
            }

            #[cfg(debug_assertions)]
            {
                self.iteration += 1;
                if start_time.elapsed() > Duration::from_millis(1000) {
                    eprintln!("WARNING: Iteration {} took longer than 1 second.", self.iteration);
                }
            }

            // Adjust sleep duration to maintain 1 second interval
            let now_sub_ms = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_millis()
                % 1000;
            if now_sub_ms < 1000 {
                thread::sleep(Duration::from_millis(1000 - now_sub_ms as u64));
            }
        }
    }
}

/// Returns whether the current process has elevated/RAPL privileges.
fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        let admin = is_admin::is_admin();
        #[cfg(debug_assertions)]
        if !admin {
            eprintln!("\u{26a0} Running without Administrator privileges. CPU power readings will use estimation.");
        }
        admin
    }

    #[cfg(target_os = "linux")]
    {
        let rapl_accessible = std::fs::read_to_string("/sys/class/powercap/intel-rapl:0/energy_uj").is_ok();
        #[cfg(debug_assertions)]
        if !rapl_accessible {
            eprintln!("\u{26a0} RAPL not accessible. CPU power readings will use estimation.");
            eprintln!("  Tip: run as root or grant read access to /sys/class/powercap/");
        }
        rapl_accessible
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        #[cfg(debug_assertions)]
        eprintln!("\u{26a0} No privileged power-reading support on this platform. Using estimation.");
        false
    }
}
