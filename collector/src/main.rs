#![allow(dead_code, unused_imports)]

use std::{
    os::raw::{c_uint, c_ulonglong, c_void},
    ptr,
    sync::mpsc,
    thread,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Utc};
use collector::{
    database::Database,
    sensors::{self, Sensor, cpu, gpu},
};
use rusqlite::{Connection, Result, ToSql};
use tray_icon::{
    Icon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, MenuItemKind},
};

use hardware_query::HardwareInfo;

fn main() {
    check_permissions();
 
    println!("\n========== SYSTEM HARDWARE INFORMATION ==========\n");
    
    // Get complete system information
    let hw_info = HardwareInfo::query().unwrap();

    // ===== CPU INFORMATION =====
    println!("===== CPU INFORMATION =====");
    let cpu = hw_info.cpu();
    println!("CPU: {} {} - {} cores, {} threads",
        cpu.vendor(),
        cpu.model_name(),
        cpu.physical_cores(),
        cpu.logical_cores()
    );

    if cpu.has_feature("avx2") && cpu.has_feature("fma") {
        println!("CPU optimized for SIMD operations");
    }

    // ===== GPU INFORMATION =====
    println!("\n===== GPU INFORMATION =====");
    for gpu in hw_info.gpus() {
        println!("GPU: {} {} - {} GB VRAM", 
            gpu.vendor(), gpu.model_name(), gpu.memory_gb());
        
        if gpu.supports_cuda() {
            println!("  CUDA support available");
        }
        if gpu.supports_opencl() {
            println!("  OpenCL support available");
        }
    }

    // ===== MEMORY INFORMATION =====
    println!("\n===== MEMORY INFORMATION =====");
    let memory = hw_info.memory();
    println!("Memory: {} GB total, {} GB available",
        memory.total_gb(),
        memory.available_gb()
    );

    // ===== THERMAL INFORMATION (FANS) =====
    println!("\n===== THERMAL INFORMATION =====");
    let thermal = hw_info.thermal();
    println!("Temperature Sensors:");
    for sensor in thermal.sensors() {
        println!("  {}: {:.1}°C", sensor.name, sensor.temperature);
    }
    
    println!("\nFan Information:");
    for fan in thermal.fans() {
        println!("  {}: {} % of speed", fan.name, fan.speed_percent.unwrap());
    }

    // ===== BATTERY INFORMATION =====
    println!("\n===== BATTERY INFORMATION =====");
    if let Some(battery) = hw_info.battery() {
        println!("Charge: {}%", battery.percentage());
        println!("Status: {:?}", battery.status());
    } else {
        println!("No battery detected (Desktop system)");
    }

    // ===== USB DEVICES =====
    println!("\n===== USB DEVICES =====");
    let usb_devices = hw_info.usb_devices();
    println!("Found {} USB devices:", usb_devices.len());
    for (i, device) in usb_devices.iter().enumerate() {
        println!("  [{}] {} - {}", i + 1, device.device_class, device.connected);
    }

    // ===== POWER SETTINGS =====
    println!("\n===== POWER SETTINGS =====");
    let power_profile = hw_info.power_profile();
    println!("Active Power Profile: {:?}", power_profile);

    // ===== AI ACCELERATORS =====
    if !hw_info.npus().is_empty() {
        println!("\n===== AI ACCELERATORS =====");
        println!("AI accelerators found: {} NPUs", hw_info.npus().len());
        for npu in hw_info.npus() {
            println!("  NPU: {} {}", npu.vendor(), npu.model_name());
        }
    }
}

// fn main() {
//     check_permissions();

//     // Initialize hardware informations
//     println!("\nDETECTED HARDWARE:\n");

//     println!();

//     println!("INITIALIZING DRIVERS:\n");
//     let sensor_cpu = sensors::cpu::get_cpu_power_sensor(0);
//     match &sensor_cpu {
//         Ok(_) => println!("CPU Power Sensor initialized successfully."),
//         Err(e) => println!("Failed to initialize CPU Power Sensor: {:?}", e),
//     }

//     println!();

//     println!("POWER CONSUMPTION MONITORING:");
//     loop {
//         thread::sleep(Duration::from_millis(1000));

//         println!("\nCPU :");
//         match sensor_cpu.as_ref().unwrap().read_full_data() {
//             Ok(event) => {
//                 println!(
//                     "Power Consumption PKG: {:.3} W",
//                     event.data().total_power_watts.unwrap_or(-1.0)
//                 );
//                 println!(
//                     "Power Consumption PP0: {:.3} W",
//                     event.data().pp0_power_watts.unwrap_or(-1.0)
//                 );
//                 println!(
//                     "Power Consumption PP1: {:.3} W",
//                     event.data().pp1_power_watts.unwrap_or(-1.0)
//                 );
//                 println!(
//                     "Power Consumption DRAM: {:.3} W",
//                     event.data().dram_power_watts.unwrap_or(-1.0)
//                 );
//                 println!("CPU Usage: {:.2} %", event.data().usage_percent);
//             }
//             Err(e) => {
//                 println!("Error reading CPU data: {:?}", e);
//             }
//         }

//             let current = gpu::GPUVendor::from_str(&gpu);
//             if prev.differs(current) && current != gpu::GPUVendor::Other {
//                 println!("\nGPU :");

//                 let sensor_gpu = sensors::gpu::get_gpu_power_sensor(&gpu, index);
//                 match sensor_gpu {
//                     Ok(sensor) => match sensor.read_full_data() {
//                         Ok(event) => {
//                             println!(
//                                 "Power Consumption: {:.3} W",
//                                 event.data().total_power_watts.unwrap_or(-1.0)
//                             );
//                             println!("GPU Usage: {:.2} %", event.data().usage_percent.unwrap_or(-1.0));
//                             println!("VRAM Usage: {:.2} %", event.data().vram_usage_percent.unwrap_or(-1.0));
//                         }
//                         Err(e) => {
//                             println!("Error reading GPU data: {:?}", e);
//                         }
//                     },
//                     Err(e) => {
//                         println!("Error initializing GPU sensor: {:?}", e);
//                     }
//                 }

//                 index = 0;
//             }
//             prev = gpu::GPUVendor::from_str(&gpu);
//             index += 1;
//         }
//     }
// }

// use display_info::DisplayInfo;
// use std::time::Instant;

// fn main() {
//   let start = Instant::now();

//   let display_infos = DisplayInfo::all().unwrap();
//   for display_info in display_infos {
//     println!("display_info {display_info:?}");
//   }
//   let display_info = DisplayInfo::from_point(100, 100).unwrap();
//   println!("display_info {display_info:?}");
//   println!("运行耗时: {:?}", start.elapsed());
// }

// // pub fn main() {
//     // let conn = Connection::open("test.db").unwrap();
//     // conn.execute(
//     //     "CREATE TABLE IF NOT EXISTS cpu_data (
//     //               id             INTEGER PRIMARY KEY,
//     //               timestamp      TEXT NOT NULL,
//     //               power_watts    REAL NOT NULL,
//     //               usage_percent  REAL NOT NULL
//     //               )",
//     //     [],
//     // ).unwrap();
//     // let sensor = sensors::cpu::get_cpu_power_sensor().unwrap();
//     // loop {
//     //     let power = sensor.read_power_watts().unwrap();
//     //     let usage = sensor.read_usage_percent().unwrap();
//     //     let mut stmt = conn.prepare("INSERT INTO cpu_data (timestamp, power_watts, usage_percent) VALUES (?1, ?2, ?3)").unwrap();
//     //     stmt.execute((DateTime::<Utc>::from(power.time()), power.data(), usage.data())).unwrap();
//     //     println!("Inserted data: time {:?}, power {:.3} W, usage {:.2} %", power.time().duration_since(UNIX_EPOCH).unwrap().as_secs_f64().to_string(), power.data(), usage.data());
//     //     println!("DateTime from SystemTime: {}", DateTime::<Utc>::from(power.time()));
//     //     let mut stmt = conn.prepare("SELECT timestamp FROM cpu_data ORDER BY id DESC LIMIT 1").unwrap();
//     //     let date_from_db: DateTime::<Utc> = stmt.query_row([], |row| {
//     //         row.get(0)
//     //     }).unwrap();
//     //     println!("DateTime from DB: {}", date_from_db);
//     //     thread::sleep(Duration::from_secs(1));
//     // }

//     // let database = Database::new("test.db").unwrap();
//     // database.create_tables_if_not_exists().unwrap();
//     // let sensor = sensors::cpu::get_cpu_power_sensor().unwrap();
//     // let gpu_sensor = sensors::gpu::get_gpu_power_sensor().unwrap();

//     // let icon = Icon::from_rgba(vec![0, 0, 0, 0], 1, 1).expect("Failed to create icon");
//     // let menu = Menu::new();
//     // let item1 = MenuItem::new("item1", true, None);
//     // menu.append(&item1).unwrap();

//     // let tray_icon = TrayIconBuilder::new()
//     //     .with_tooltip("PC Power Collector")
//     //     .with_icon(icon)
//     //     .with_menu(Box::new(menu))
//     //     .build()
//     //     .expect("Failed to create tray icon");

//     // loop {
//     //     let event = match sensor.read_full_data() {
//     //         Ok(event) => event,
//     //         Err(e) => {
//     //             eprintln!("Error reading CPU data: {:?}", e);
//     //             continue;
//     //         }
//     //     };
//     //     database.insert_cpu_data(&event).unwrap();
//     //     println!("Read CPU data: power {:.3} W, usage {:.2} %", event.data().total_power_watts, event.data().usage_percent);

//     //     tray_icon.set_tooltip(Some(format!("CPU Power: {:.3} W, Usage: {:.2} %", event.data().total_power_watts, event.data().usage_percent))).unwrap();
//     //     let gpu_event = gpu_sensor.read_full_data().unwrap();
//     //     println!("GPU power {:.3} W, CPU power {:.3} W, 2 RAMS : 10 W, total {:.3} W", gpu_event.data().total_power_watts, event.data().total_power_watts, gpu_event.data().total_power_watts + event.data().total_power_watts + 10.0);
//     //     thread::sleep(Duration::from_millis(1000));
//     // }
//     // let (tx, rx) = mpsc::channel();
//     // let tx2 = tx.clone();
//     // thread::spawn(move || {
//     //     let vals = vec![
//     //         String::from("hi"),
//     //         String::from("from"),
//     //         String::from("the"),
//     //         String::from("thread"),
//     //     ];
//     //     for val in vals {
//     //         tx.send(val).unwrap();
//     //         thread::sleep(Duration::from_secs(1));
//     //     }
//     // });

//     // thread::spawn(move || {
//     //     let vals = vec![
//     //         String::from("more"),
//     //         String::from("messages"),
//     //         String::from("for"),
//     //         String::from("you"),
//     //     ];
//     //     for val in vals {
//     //         tx2.send(val).unwrap();
//     //         thread::sleep(Duration::from_secs(1));
//     //     }
//     // });

//     // for received in rx {
//     //     println!("Got: {}", received);
//     // }

//     // check_permissions();
//     // let sensor = sensors::cpu::get_cpu_power_sensor().unwrap();
//     // for _ in 0..5 {
//     //     println!("Using sensor: {}", sensor.name());
//     //     let power = sensor.read_power_watts().unwrap();
//     //     println!("CPU Power: {:.3} W at time {:?}", power.value(), power.time().duration_since(UNIX_EPOCH).unwrap());
//     //     let usage = sensor.read_usage_percent().unwrap();
//     //     println!("CPU Usage: {:.2} % at time {:?}", usage.value(), usage.time().duration_since(UNIX_EPOCH).unwrap());
//     //     thread::sleep(Duration::from_secs(1));
//     // }
// // }

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
            eprintln!("Please run with: sudo {}", std::env::current_exe().unwrap().display());
            std::process::exit(1);
        }
    }
}
