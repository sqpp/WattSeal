use win_ring0::WinRing0;

pub struct WinRing0Handler {
    pub ring0: WinRing0,
}

impl WinRing0Handler {
    pub fn new() -> Result<Self, String> {
        println!("Attempting to initialize WinRing0 driver...");
        let mut handler = WinRing0Handler { ring0: WinRing0::new() };
        println!("Installing WinRing0 driver...");
        handler.ring0.install()?;
        println!("Opening WinRing0 driver...");
        handler.ring0.open()?;
        println!("WinRing0 driver initialized and opened successfully.");
        Ok(handler)
    }
}

impl Drop for WinRing0Handler {
    fn drop(&mut self) {
        println!("Closing WinRing0 driver");
        self.ring0.close();
        println!("Uninstalling WinRing0 driver");
        self.ring0.uninstall();
    }
}
