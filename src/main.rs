#![allow(dead_code, unused_imports)]

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
};

use collector::CollectorApp;
use tray_icon::{
    TrayIconBuilder, TrayIconEvent,
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

fn main() {
    let should_quit = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        println!("Starting collector...");
        let mut app = CollectorApp::new().expect("Failed to create CollectorApp");

        if let Err(e) = app.initialize() {
            eprintln!("Failed to initialize collector: {}", e);
            return;
        }
        tx.send(()).unwrap_or_default();
        app.run();
    });

    let icon = tray_icon::Icon::from_rgba(vec![0, 255, 0, 0], 1, 1).expect("Failed to create icon");

    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quit", true, None);
    let quit_id = quit_i.id().to_owned();

    tray_menu
        .append_items(&[
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

    let should_quit_menu = should_quit.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == quit_id {
            should_quit_menu.store(true, Ordering::Relaxed);
            std::process::exit(0);
        }
    }));

    TrayIconEvent::set_event_handler(Some(|event| match event {
        TrayIconEvent::DoubleClick { .. } => {
            println!("Double clicked tray icon");
        }
        _ => {}
    }));

    let _ = rx.recv();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("WattAware")
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon");

    println!("Starting UI...");
    if let Err(e) = ui::run() {
        eprintln!("UI error: {}", e);
    }

    should_quit.store(true, Ordering::Relaxed);
}
