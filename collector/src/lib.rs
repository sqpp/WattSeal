pub mod database;
pub mod sensors;

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

/// In debug → `println!`.  In release → append timestamped line to log file.
macro_rules! clog {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        println!($($arg)*);
        #[cfg(not(debug_assertions))]
        log_to_file(&format!($($arg)*));
    }};
}

#[cfg(not(debug_assertions))]
const LOG_FILE: &str = "collector.log";

/// Written once at the start of every init session.
#[cfg(not(debug_assertions))]
const SESSION_MARKER: &str = ">>> session start <<<";

/// Append a timestamped line to the log file.
#[cfg(not(debug_assertions))]
fn log_to_file(msg: &str) {
    use std::{fs::OpenOptions, io::Write};
    let Ok(mut f) = OpenOptions::new().create(true).append(true).open(LOG_FILE) else {
        return;
    };
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let _ = writeln!(f, "[{ts}] {msg}");
}

/// Trim `collector.log` so that only the last 2 sessions remain, then append the session marker.
#[cfg(not(debug_assertions))]
fn start_log_session() {
    use std::io::BufRead;
    if let Ok(f) = std::fs::File::open(LOG_FILE) {
        let lines: Vec<String> = std::io::BufReader::new(f).lines().flatten().collect();
        let starts: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.contains(SESSION_MARKER))
            .map(|(i, _)| i)
            .collect();
        if starts.len() >= 2 {
            let keep_from = starts[starts.len() - 2];
            let _ = std::fs::write(LOG_FILE, lines[keep_from..].join("\n") + "\n");
        }
    }
    log_to_file(SESSION_MARKER);
}

pub struct CollectorApp {
    database: Database,
    sensors: Vec<SensorType>,
    system: Rc<RefCell<System>>,
    last_update: Instant,
    #[cfg(debug_assertions)]
    iteration: u64,
}

impl CollectorApp {
    pub fn new() -> Result<Self, String> {
        let database = Database::new().map_err(|e| format!("Failed to create database: {e}"))?;
        let s = System::new_all();
        Ok(CollectorApp {
            database,
            sensors: Vec::new(),
            system: Rc::new(RefCell::new(s)),
            last_update: Instant::now(),
            #[cfg(debug_assertions)]
            iteration: 0,
        })
    }

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

        clog!("Initialization complete");
        Ok(())
    }

    pub fn run(&mut self) {
        // Purge/averaging runs in a separate thread so collection starts immediately.
        thread::spawn(|| {
            if let Ok(mut db) = Database::new() {
                let _ = averaging_and_purging_data(&mut db, 24, 24);
            }
        });

        #[cfg(debug_assertions)]
        println!(
            "\n========== POWER CONSUMPTION MONITORING ==========\nLogging to database every second. Press Ctrl+C to stop.\n"
        );

        loop {
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
