use std::net::TcpListener;

/// Port used as an inter-process mutex to prevent duplicate collector instances.
const SINGLETON_PORT: u16 = 47285;

/// Holds a [`TcpListener`] bound to a fixed local port.
///
/// While alive, no other process can bind the same port, which acts as a
/// cross-platform single-instance guard.  When the guard is dropped (or the
/// process exits), the OS releases the port automatically.
pub struct SingletonGuard {
    _listener: TcpListener,
}

impl SingletonGuard {
    /// Try to acquire the singleton lock.
    ///
    /// Returns `Ok(guard)` if this is the only running instance, or an error
    /// message if another instance already holds the port.
    pub fn acquire() -> Result<Self, String> {
        TcpListener::bind(("127.0.0.1", SINGLETON_PORT))
            .map(|l| SingletonGuard { _listener: l })
            .map_err(|_| {
                "Another instance of WattSeal is already running. \
                 Exiting to prevent database corruption."
                    .to_string()
            })
    }
}
