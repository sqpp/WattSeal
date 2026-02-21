pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod network;
pub mod process;
pub mod ram;

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Instant, SystemTime},
};

use adlx::system;
use battery::{Battery, Manager};
pub use common::{
    Event, ProcessData, SensorData, TotalData,
    types::{BatteryInfo, CpuInfo, DiskInfo, HardwareInfo, MemoryInfo, MotherboardInfo, ScreenInfo, SystemInfo},
};
pub use cpu::CPUSensor;
pub use disk::DiskSensor;
use display_info::DisplayInfo;
pub use gpu::{GPUSensor, get_gpu_list};
pub use network::NetworkSensor;
pub use process::get_processes;
pub use ram::RamSensor;
use sysinfo::{Components, CpuRefreshKind, Disks, Networks, System};

pub enum SensorType {
    CPU(CPUSensor),
    GPU(GPUSensor),
    RAM(RamSensor),
    Disk(DiskSensor),
    Network(NetworkSensor),
    Process,
    Total,
}

impl Sensor for SensorType {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        match self {
            SensorType::CPU(sensor) => sensor.read_full_data(),
            SensorType::GPU(sensor) => sensor.read_full_data(),
            SensorType::RAM(sensor) => sensor.read_full_data(),
            SensorType::Disk(sensor) => sensor.read_full_data(),
            SensorType::Network(sensor) => sensor.read_full_data(),
            SensorType::Process => Err(SensorError::NotSupported),
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

pub fn create_event_from_sensors(sensors: &Vec<SensorType>, system: Rc<RefCell<System>>, all_time: &mut AllTimeData) -> Event {
    let time = SystemTime::now();
    let mut data: Vec<SensorData> = Vec::new();

    let (mut cpu_power, mut cpu_usage, mut nb_cpus) = (0.0, 0.0, 0);
    let (mut gpu_power, mut gpu_usage, mut nb_gpus) = (0.0, 0.0, 0);

    let mut total_power = 0.0;
    let mut proc_gpu_usage = HashMap::new();
    for sensor in sensors {
        match sensor {
            SensorType::Process | SensorType::Total => continue,
            SensorType::GPU(gpu_sensor) => {
                if let Ok(gpu_process_usage) = gpu_sensor.get_process_gpu_usage(
                    time.duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                ) {
                    proc_gpu_usage.extend(gpu_process_usage);
                }
            }
            _ => {}
        }
        if let SensorType::Total | SensorType::Process = sensor {
            continue;
        }

        let sensor_data = sensor.read_full_data();
        match sensor_data {
            Ok(d) => {
                if let Some(power) = d.total_power_watts() {
                    total_power += power;

                    if let SensorData::CPU(cpu) = &d {
                        cpu_power += power;
                        cpu_usage += cpu.usage_percent.unwrap_or(0.0);
                        nb_cpus += 1;
                    }

                    if let SensorData::GPU(gpu) = &d {
                        gpu_power += power;
                        gpu_usage += gpu.usage_percent.unwrap_or(0.0);
                        nb_gpus += 1;
                    }
                }
                data.push(d);
            }
            Err(e) => eprintln!("✗ Error reading sensor data: {:?}", e),
        }
    }
    data.push(SensorData::Total(TotalData {
        total_power_watts: total_power,
        period_type: "second".to_string(),
    }));

    all_time.update(total_power);

    cpu_usage /= nb_cpus.max(1) as f64;
    gpu_usage /= nb_gpus.max(1) as f64;

    let timer = Instant::now();
    let top10_process_data: Vec<ProcessData> = get_processes(
        system.clone(),
        cpu_power,
        cpu_usage,
        gpu_power,
        gpu_usage,
        total_power,
        10,
        proc_gpu_usage,
    );
    data.push(SensorData::Process(top10_process_data));
    println!("Process time: {} ms", timer.elapsed().as_millis());

    return Event::new(time, data);
}

pub fn get_hardware_info(system: Rc<RefCell<System>>) -> HardwareInfo {
    let sys = system.borrow_mut();

    // Cpu information
    // sys.refresh_cpu_list(CpuRefreshKind::everything().without_cpu_usage());
    let logical_cores = sys.cpus().len() as u16;
    let physical_cores = System::physical_core_count().unwrap_or(0) as u16;
    let cpu_name = sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());
    let cpu_vendor = sys
        .cpus()
        .first()
        .map(|cpu| cpu.vendor_id().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_frequency = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);

    let cpu_info = CpuInfo {
        name: cpu_name,
        vendor: cpu_vendor,
        base_frequency_mhz: cpu_frequency,
        logical_cores,
        physical_cores,
    };

    // Memory information
    let total_ram = sys.total_memory();
    let total_swap = sys.total_swap();

    let memory_info = MemoryInfo {
        total_ram_bytes: total_ram,
        total_swap_bytes: total_swap,
    };

    // System information
    let os_name = format!(
        "{} {}",
        System::name().unwrap_or_default(),
        System::os_version().unwrap_or_default()
    );
    let hostname = System::host_name().unwrap_or_default();
    let architecture = std::env::consts::ARCH.to_string();

    let system_info = SystemInfo {
        os: os_name,
        hostname,
        architecture,
        is_virtual_machine: false,
    };

    // Disks information
    let disks = Disks::new_with_refreshed_list();
    let mut disk_infos = Vec::new();

    for disk in &disks {
        let name = disk.name().to_string_lossy().to_string();
        let mount = disk.mount_point().display().to_string();
        let kind = disk_kind_label(disk);
        let total = disk.total_space();
        let avail = disk.available_space();
        let used = total - avail;
        let fs = disk.file_system().to_string_lossy().to_string();

        disk_infos.push(DiskInfo {
            name: name.clone(),
            mount_point: mount.clone(),
            file_system: fs,
            disk_type: kind.to_string(),
            total_bytes: total,
            used_bytes: used,
        });
    }

    // Display info
    let display_infos = DisplayInfo::all().unwrap();
    let mut screen_infos = Vec::new();
    for display_info in display_infos {
        let resolution = format!("{}x{}", display_info.width, display_info.height);

        screen_infos.push(ScreenInfo {
            model: display_info.friendly_name,
            resolution: resolution,
            refresh_rate_hz: display_info.frequency as u32,
            is_primary: display_info.is_primary,
        });
    }

    // Battery info

    let battery_info = BatteryInfo {
        present: false,
        design_capacity_wh: None,
        full_charge_capacity_wh: None,
        cycle_count: None,
    };

    let manager = Manager::new().unwrap();
    let battery_info = match manager.batteries() {
        Ok(mut batteries) => {
            if let Some(Ok(battery)) = batteries.next() {
                BatteryInfo {
                    present: true,
                    design_capacity_wh: Some(battery.energy_full_design().get::<battery::units::energy::watt_hour>()),
                    full_charge_capacity_wh: Some(battery.energy_full().get::<battery::units::energy::watt_hour>()),
                    cycle_count: battery.cycle_count(),
                }
            } else {
                battery_info
            }
        }
        Err(_) => battery_info,
    };

    // Build the HardwareInfo structure
    let hardware_info = HardwareInfo {
        system: system_info,
        cpu: cpu_info,
        memory: memory_info,
        motherboard: MotherboardInfo {
            manufacturer: "Unknown".to_string(),
            model: "Unknown".to_string(),
            serial: "Unknown".to_string(),
        },
        gpus: get_gpu_list(),
        disks: disk_infos,
        displays: screen_infos,
        battery: battery_info,
    };

    return hardware_info;
}

fn disk_kind_label(kind: &sysinfo::Disk) -> &'static str {
    if kind.is_removable() {
        "Removable"
    } else {
        match kind.kind() {
            sysinfo::DiskKind::HDD => "HDD",
            sysinfo::DiskKind::SSD => "SSD",
            _ => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AllTimeData {
    pub total_power_watts: f64,
    pub duration_seconds: u64,
}

impl AllTimeData {
    pub fn new() -> Self {
        AllTimeData {
            total_power_watts: 0.0,
            duration_seconds: 0,
        }
    }

    pub fn update(&mut self, power_watts: f64) {
        self.total_power_watts += power_watts;
        self.duration_seconds += 1;
    }

    pub fn average_power(&self) -> f64 {
        if self.duration_seconds > 0 {
            self.total_power_watts / self.duration_seconds as f64
        } else {
            0.0
        }
    }
}
