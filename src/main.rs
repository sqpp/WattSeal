// 

use is_admin;

pub fn main() {
    // Check if we have the required privileges
    check_permissions();

    // let mut r0: Box<WinRing0> = Box::from(WinRing0::new());

    // println!("Installing ring0 driver");
    // match r0.install() {
    //     Ok(()) => {
    //         println!("Driver installed");
    //     }
    //     Err(err) => {
    //         println!("Error: {}", err);
    //     }
    // }

    // println!("Opening ring0 driver");
    // match r0.open() {
    //     Ok(()) => {
    //         println!("Driver opened");
    //     }
    //     Err(err) => {
    //         println!("Error: {}", err);
    //     }
    // }

    // println!("Trying to get tjMax value, should work on most Intel CPUs");
    // // MSR_TEMPERATURE_TARGET
    // let msr = 0x1a2;
    // match r0.readMsr(msr) {
    //     Ok(out) => {
    //         let _edx = ((out >> 32) & 0xffffffff) as u32;
    //         let eax = (out & 0xffffffff) as u32;
    //         let tj_max = (eax >> 16) & 0xff;
    //         println!("MSR Value: {}", tj_max);
    //     }
    //     Err(err) => {
    //         println!("Error reading MSR: {}", err);
    //     }
    // }

    // // MSR_RAPL_POWER_UNIT
    // let msr = 0x606;
    // match r0.readMsr(msr) {
    //     Ok(out) => {
    //         // Split the 64 bit value into the original by offsetting and masking
    //         // EDX: high order 32 bits
    //         let _edx = ((out >> 32) & 0xffffffff) as u32;
    //         // EAX: low order 32 bits
    //         let eax = (out & 0xffffffff) as u32;
    //         // power_unit = 1/2^PU where PU is bits 3:0 of EAX
    //         let power_unit = 1.0 / f64::from(1 << (eax & 0xf));
    //         // energy_unit = 1/2^EU where EU is bits 12:8 of EAX
    //         let energy_unit = 1.0 / f64::from(1 << ((eax >> 8) & 0x1f));
    //         // time_unit = 1/2^TU where TU is bits 19:16 of EAX
    //         let time_unit = 1.0 / f64::from(1 << ((eax >> 16) & 0xf));

    //         println!("Raw Power Unit: {:b}", eax & 0xf);
    //         println!("Raw Energy Unit: {:b}", (eax >> 8) & 0x1f);
    //         println!("Raw Time Unit: {:b}", (eax >> 16) & 0xf);

    //         println!("Power Unit: {}mW", power_unit);
    //         println!("Energy Unit: {}µJ", energy_unit);
    //         println!("Time Unit: {}s", time_unit);
    //     }
    //     Err(err) => {
    //         println!("Error reading MSR: {}", err);
    //     }
    // }

    // // AMD ENERGY_PWR_UNIT_MSR
    // let msr = 0xC0010299;
    // match r0.readMsr(msr) {
    //     Ok(out) => {
    //         let _edx = ((out >> 32) & 0xffffffff) as u32;
    //         let eax = (out & 0xffffffff) as u32;
    //         let energy_unit = 1.0 / f64::from(1 << (eax & 0xf));

    //         println!("AMD Raw Energy Unit: {:b}", eax & 0xf);
    //         println!("AMD Energy Unit: {}µJ", energy_unit);
    //     }
    //     Err(err) => {
    //         println!("Error reading MSR: {}", err);
    //     }
    // }

    // // AMD ENERGY_CORE_MSR
    // let msr = 0xC001029A;
    // match r0.readMsr(msr) {
    //     Ok(out) => {
    //         let edx = ((out >> 32) & 0xffffffff) as u32;
    //         let eax = (out & 0xffffffff) as u32;
    //         let energy = ((edx as u64) << 32) | (eax as u64);
    //         println!("AMD Core Energy: {}", energy);
    //     }
    //     Err(err) => {
    //         println!("Error reading MSR: {}", err);
    //     }
    // }
    // uninstall_driver(r0);
}

// fn uninstall_driver(mut r0: Box<WinRing0>) {
//     println!("Closing ring0 driver");
//     match r0.close() {
//         Ok(()) => {
//             println!("Driver closed");
//         }
//         Err(err) => {
//             println!("Error: {}", err);
//         }
//     }

//     println!("Uninstall ring0 driver");
//     match r0.uninstall() {
//         Ok(()) => {
//             println!("Driver uninstalled");
//         }
//         Err(err) => {
//             println!("Error: {}", err);
//         }
//     }
// }

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
