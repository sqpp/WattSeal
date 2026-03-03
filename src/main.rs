#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    process::{Child, Command},
    sync::{Arc, Mutex, mpsc},
    thread,
};

use collector::CollectorApp;
use tray_icon::{
    TrayIconBuilder, TrayIconEvent,
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

fn spawn_ui(ui_child: &Mutex<Option<Child>>) -> Result<(), String> {
    let mut guard = ui_child
        .lock()
        .map_err(|e| format!("Failed to lock UI child mutex: {}", e))?;
    let already_running = guard.as_mut().is_some_and(|c| matches!(c.try_wait(), Ok(None)));
    if already_running {
        return Ok(());
    }
    if let Ok(exe) = std::env::current_exe() {
        *guard = Command::new(exe).arg("--ui").spawn().ok();
    }
    Ok(())
}

fn main() {
    #[cfg(target_os = "windows")]
    if !std::env::args().any(|a| a == "--ui") && !is_admin::is_admin() {
        let exe = std::env::current_exe();
        if let Ok(exe) = exe {
            let args: Vec<String> = std::env::args().skip(1).collect();
            let relaunched = runas::Command::new(&exe).args(&args).gui(true).status();
            match relaunched {
                Ok(status) if status.success() => return,
                _ => {}
            }
        }
    }

    if std::env::args().any(|a| a == "--ui") {
        let _ = ui::run();
        return;
    }

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut app = match CollectorApp::new() {
            Ok(app) => app,
            Err(_) => return,
        };
        if app.initialize().is_err() {
            return;
        }
        tx.send(()).unwrap_or_default();
        app.run();
    });

    // Wait for collector to finish initializing
    let _ = rx.recv();

    // Create event loop for tray icon
    let event_loop = match EventLoop::new() {
        Ok(loop_handle) => loop_handle,
        Err(_) => return,
    };

    // Build tray menu
    let tray_menu = Menu::new();
    let open_ui_i = MenuItem::new("Open UI", true, None);
    let quit_i = MenuItem::new("Quit", true, None);
    let open_ui_id = open_ui_i.id().to_owned();
    let quit_id = quit_i.id().to_owned();

    tray_menu
        .append_items(&[
            &open_ui_i,
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("WattAware".to_string()),
                    copyright: Some("Copyright 2026".to_string()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &quit_i,
        ])
        .ok();

    let ui_child: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    let ui_child_menu = Arc::clone(&ui_child);
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == open_ui_id {
            spawn_ui(&ui_child_menu).ok();
        } else if event.id == quit_id {
            if let Ok(mut child_guard) = ui_child_menu.lock() {
                if let Some(c) = child_guard.as_mut() {
                    let _ = c.kill();
                }
            }
            std::process::exit(0);
        }
    }));

    let ui_child_tray = Arc::clone(&ui_child);
    TrayIconEvent::set_event_handler(Some(move |event| {
        if let TrayIconEvent::DoubleClick { .. } = event {
            spawn_ui(&ui_child_tray).ok();
        }
    }));

    let icon = tray_icon::Icon::from_rgba(vec![0, 255, 0, 0], 1, 1).ok();

    let _tray_icon = icon.and_then(|icon| {
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("WattAware")
            .with_icon(icon)
            .build()
            .ok()
    });

    spawn_ui(&ui_child).ok();

    // Run the event loop (pumps Windows messages for tray icon)
    struct TrayApp;
    impl ApplicationHandler for TrayApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
        fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}
    }
    event_loop.run_app(&mut TrayApp).ok();
}
