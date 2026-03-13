use std::{any::Any, ffi::c_ulong, panic, process::Command};

use common::clog;
use win_ring0::WinRing0;

/// Safe wrapper around the WinRing0 kernel driver for MSR access.
pub struct WinRing0Reader {
    ring0: WinRing0,
}

impl WinRing0Reader {
    /// Installs and opens the WinRing0 driver, recovering from stuck state if needed.
    pub fn new() -> Result<Self, String> {
        clog!("Initializing WinRing0 driver...");
        let mut handler = match panic::catch_unwind(|| WinRing0Reader { ring0: WinRing0::new() }) {
            Ok(handler) => handler,
            Err(e) => Self::free_stuck_driver(e)?,
        };
        clog!("Installing WinRing0 driver...");
        handler.ring0.install().or_else(|_| handler.retry_install())?;
        clog!("Opening WinRing0 driver...");
        handler.ring0.open()?;
        clog!("WinRing0 driver opened successfully.");
        Ok(handler)
    }

    /// Reads a Model-Specific Register by address.
    pub fn read_msr(&self, msr: c_ulong) -> Result<u64, String> {
        self.ring0.readMsr(msr)
    }

    /// Uninstalls and re-installs the driver after a failed install.
    fn retry_install(&mut self) -> Result<(), String> {
        clog!("Uninstalling existing WinRing0 driver...");
        self.ring0.uninstall()?;
        clog!("Retrying WinRing0 driver installation...");
        self.ring0.install()?;
        Ok(())
    }

    /// Stops a stuck WinRing0 service and re-creates the reader.
    fn free_stuck_driver(_: Box<dyn Any + Send>) -> Result<Self, String> {
        clog!("WinRing0 initialization panicked. Freeing stuck driver...");
        // sc stop WinRing0_1_2_0
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
        let sc_path = format!(r"{}\System32\sc.exe", system_root);
        Command::new(&sc_path).args(["stop", "WinRing0_1_2_0"]).output().ok();
        clog!("Stuck WinRing0 driver stopped successfully.");
        let mut handler = panic::catch_unwind(|| WinRing0Reader { ring0: WinRing0::new() })
            .map_err(|e| format!("Failed to create WinRing0Reader after freeing driver: {:?}", e))?;
        match handler.ring0.uninstall() {
            Ok(_) => clog!("Stuck WinRing0 driver uninstalled successfully."),
            Err(e) => clog!("Error uninstalling stuck WinRing0 driver: {}", e),
        }
        return Ok(handler);
    }
}

impl Drop for WinRing0Reader {
    fn drop(&mut self) {
        clog!("Closing WinRing0 driver...");
        match self.ring0.close() {
            Ok(_) => clog!("WinRing0 driver closed successfully."),
            Err(e) => clog!("Error closing WinRing0 driver: {}", e),
        }
        clog!("Uninstalling WinRing0 driver...");
        match self.ring0.uninstall() {
            Ok(_) => clog!("WinRing0 driver uninstalled successfully."),
            Err(e) => clog!("Error uninstalling WinRing0 driver: {}", e),
        }
    }
}
