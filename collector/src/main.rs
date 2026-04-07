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

    let args: Vec<String> = std::env::args().skip(1).collect();
    let power_log_path = args
        .iter()
        .position(|a| a == "--power-log")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let mut app = match CollectorApp::new() {
        Ok(mut app) => {
            if let Some(path) = power_log_path {
                app = app.with_power_log(path);
            }
            app
        }
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
