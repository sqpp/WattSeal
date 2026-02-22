use std::{cell::RefCell, collections::HashMap, hash::Hash, time::Instant};

use common::types::{DiskInfo, InitialInfo};
use sysinfo::Disks;

use crate::{
    database::{DiskData, SensorData},
    sensors::{Sensor, SensorError},
};

pub struct DiskSensor {
    disks: RefCell<Disks>,
}

impl DiskSensor {
    pub fn new() -> Self {
        Self {
            disks: RefCell::new(Disks::new_with_refreshed_list()),
        }
    }
}

impl Sensor for DiskSensor {
    fn read_full_data(&self) -> Result<SensorData, SensorError> {
        // let mut total_space = 0.0;
        // let mut used_space = 0.0;
        // let mut free_space = 0.0;
        let mut read_speed = 0.0;
        let mut write_speed = 0.0;

        let mut disks = self
            .disks
            .try_borrow_mut()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow disks: {}", e)))?;
        disks.refresh(true);

        for disk in disks.iter() {
            let usage = disk.usage();
            read_speed += usage.read_bytes as f64 / 1_048_576.0; // Convert to MB/s
            write_speed += usage.written_bytes as f64 / 1_048_576.0; // Convert to MB/s
            // total_space += disk.total_space() as f64 / 1_073_741_824.0; // Convert to GB
            // used_space += (disk.total_space() - disk.available_space()) as f64 / 1_073_741_824.0; // Convert to GB
            // free_space += disk.available_space() as f64 / 1_073_741_824.0; // Convert to GB
        }

        Ok(SensorData::Disk(DiskData {
            total_power_watts: None,
            // total_gb: total_space,
            // used_gb: used_space,
            // free_gb: free_space,
            read_usage_mb_s: read_speed,
            write_usage_mb_s: write_speed,
        }))
    }

    fn read_initial_info(&self) -> Result<InitialInfo, SensorError> {
        let disks = self
            .disks
            .try_borrow()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow disks: {}", e)))?;

        let mut disk_infos = Vec::new();
        for disk in disks.list() {
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
        Ok(InitialInfo::Disks(disk_infos))
    }

    fn read_name(&self) -> Result<String, SensorError> {
        let disks = self
            .disks
            .try_borrow()
            .map_err(|e| SensorError::ReadError(format!("Failed to borrow disks: {}", e)))?;

        let names: Vec<String> = disks
            .list()
            .iter()
            .map(|disk| disk.name().to_string_lossy().to_string())
            .collect();

        Ok(format!("Disk(s): [{}]", names.join(", ")))
    }
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
