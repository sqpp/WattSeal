use std::{any::Any, ffi::c_ulong, panic, process::Command};

use win_ring0::WinRing0;

/// Safe wrapper around the WinRing0 kernel driver for MSR access.
pub struct WinRing0Reader {
    ring0: WinRing0,
}

impl WinRing0Reader {
    /// Installs and opens the WinRing0 driver, recovering from stuck state if needed.
    pub fn new() -> Result<Self, String> {
        println!("Initializing WinRing0 driver...");
        let mut handler = match panic::catch_unwind(|| WinRing0Reader { ring0: WinRing0::new() }) {
            Ok(handler) => handler,
            Err(e) => Self::free_stuck_driver(e)?,
        };
        println!("Installing WinRing0 driver...");
        handler.ring0.install().or_else(|_| handler.retry_install())?;
        println!("Opening WinRing0 driver...");
        handler.ring0.open()?;
        println!("WinRing0 driver opened successfully.");
        Ok(handler)
    }

    /// Reads a Model-Specific Register by address.
    pub fn read_msr(&self, msr: c_ulong) -> Result<u64, String> {
        self.ring0.readMsr(msr)
    }

    /// Uninstalls and re-installs the driver after a failed install.
    fn retry_install(&mut self) -> Result<(), String> {
        println!("Uninstalling existing WinRing0 driver...");
        self.ring0.uninstall()?;
        println!("Retrying WinRing0 driver installation...");
        self.ring0.install()?;
        Ok(())
    }

    /// Stops a stuck WinRing0 service and re-creates the reader.
    fn free_stuck_driver(_: Box<dyn Any + Send>) -> Result<Self, String> {
        println!("WinRing0 initialization panicked. Freeing stuck driver...");
        // sc stop WinRing0_1_2_0
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
        let sc_path = format!(r"{}\System32\sc.exe", system_root);
        Command::new(&sc_path).args(["stop", "WinRing0_1_2_0"]).output().ok();
        println!("Stuck WinRing0 driver stopped successfully.");
        let mut handler = panic::catch_unwind(|| WinRing0Reader { ring0: WinRing0::new() })
            .map_err(|e| format!("Failed to create WinRing0Reader after freeing driver: {:?}", e))?;
        match handler.ring0.uninstall() {
            Ok(_) => println!("Stuck WinRing0 driver uninstalled successfully."),
            Err(e) => println!("Error uninstalling stuck WinRing0 driver: {}", e),
        }
        return Ok(handler);
    }
}

impl Drop for WinRing0Reader {
    fn drop(&mut self) {
        println!("Closing WinRing0 driver...");
        match self.ring0.close() {
            Ok(_) => println!("WinRing0 driver closed successfully."),
            Err(e) => println!("Error closing WinRing0 driver: {}", e),
        }
        println!("Uninstalling WinRing0 driver...");
        match self.ring0.uninstall() {
            Ok(_) => println!("WinRing0 driver uninstalled successfully."),
            Err(e) => println!("Error uninstalling WinRing0 driver: {}", e),
        }
    }
}
