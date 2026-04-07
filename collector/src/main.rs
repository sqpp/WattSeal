use collector::CollectorApp;

fn main() {
    let _ = common::set_current_dir_to_exe_dir();

    let _singleton = match common::SingletonGuard::acquire(common::DATABASE_PATH) {
        Ok(guard) => guard,
        Err(msg) => {
            eprintln!("{msg}");
            return;
        }
    };

    let config = collector::config::CollectorConfig::from_args();

    let mut app = match CollectorApp::new() {
        Ok(app) => app.with_config(config),
        Err(e) => {
            eprintln!("Failed to create CollectorApp: {}", e);
            return;
        }
    };
    if let Err(e) = app.initialize() {
        eprintln!("Failed to initialize CollectorApp: {}", e);
        return;
    }
    app.run();
}
