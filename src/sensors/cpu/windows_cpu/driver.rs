use std::ffi::c_ulong;

use win_ring0::WinRing0;

pub struct WinRing0Reader {
    ring0: WinRing0,
}

impl WinRing0Reader {
    pub fn new() -> Result<Self, String> {
        println!("Attempting to initialize WinRing0 driver...");
        let mut handler = WinRing0Reader { ring0: WinRing0::new() };
        println!("Installing WinRing0 driver...");
        handler.ring0.install()?;
        println!("Opening WinRing0 driver...");
        handler.ring0.open()?;
        println!("WinRing0 driver initialized and opened successfully.");
        Ok(handler)
    }

    pub fn read_msr(&self, msr: c_ulong) -> Result<u64, String> {
        self.ring0.readMsr(msr)
    }
}

impl Drop for WinRing0Reader {
    fn drop(&mut self) {
        println!("Closing WinRing0 driver");
        match self.ring0.close() {
            Ok(_) => println!("WinRing0 driver closed successfully."),
            Err(e) => println!("Error closing WinRing0 driver: {}", e),
        }
        println!("Uninstalling WinRing0 driver");
        match self.ring0.uninstall() {
            Ok(_) => println!("WinRing0 driver uninstalled successfully."),
            Err(e) => println!("Error uninstalling WinRing0 driver: {}", e),
        }
    }
}
