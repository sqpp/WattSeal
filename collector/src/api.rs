use common::database::Database;
use common::types::{SensorData, TotalData};
use common::DatabaseEntry;
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;
use tiny_http::{Header, Response, Server, StatusCode};

pub fn start_api_server(port: u16, api_key: Option<String>) {
    let api_key = Arc::new(api_key);
    
    thread::spawn(move || {
        let server = match Server::http(format!("0.0.0.0:{}", port)) {
            Ok(server) => server,
            Err(e) => {
                common::clog!("✗ Failed to start API server: {}", e);
                return;
            }
        };

        common::clog!("✓ API Server listening on port {}", port);

        // API thread gets its own Database instance
        let mut db = match Database::new() {
            Ok(db) => db,
            Err(e) => {
                common::clog!("✗ Database error in API thread: {}", e);
                return;
            }
        };

        for request in server.incoming_requests() {
            // Check API Key Auth if configured
            if let Some(ref key) = *api_key {
                let mut authorized = false;
                for header in request.headers() {
                    if header.field.as_str().as_str().eq_ignore_ascii_case("x-api-key") {
                        if header.value.as_str() == key {
                            authorized = true;
                        }
                    }
                }
                
                // Fallback to query param auth ?api_key=
                if !authorized && request.url().contains(format!("api_key={}", key).as_str()) {
                    authorized = true;
                }

                if !authorized {
                    let response = Response::from_string("Unauthorized").with_status_code(StatusCode(401));
                    let _ = request.respond(response);
                    continue;
                }
            }

            let url = request.url().to_string();
            let response_body;
            let mut status = StatusCode(200);

            // Match simple endpoints matching app chart behaviors
            if url.starts_with("/api/stats/") {
                if url.starts_with("/api/stats/summary") {
                    // Prefer the last completed window, but if it is empty (common right after startup),
                    // fall back to the current rolling window so summary is never misleadingly 0.
                    let get_summary_avg = |db: &mut Database, period_secs: i64| -> f64 {
                        if let Ok(data) = db.select_last_n_seconds_average(
                            period_secs * 2,
                            TotalData::table_name_static(),
                            period_secs,
                        ) {
                            let values: Vec<f64> = data.into_iter().filter_map(|(_, s)| {
                                if let SensorData::Total(tot) = s {
                                    Some(tot.total_power_watts)
                                } else {
                                    None
                                }
                            }).collect();

                            if values.is_empty() {
                                return 0.0;
                            }

                            let current = *values.last().unwrap_or(&0.0);
                            let previous = if values.len() >= 2 {
                                values[values.len() - 2]
                            } else {
                                0.0
                            };

                            if previous > 0.0 { previous } else { current }
                        } else { 0.0 }
                    };

                    let last_minute = get_summary_avg(&mut db, 60);
                    let last_hour = get_summary_avg(&mut db, 3600);
                    let last_day = get_summary_avg(&mut db, 3600 * 24);
                    let last_week = get_summary_avg(&mut db, 3600 * 24 * 7);
                    let last_month = get_summary_avg(&mut db, 2592000);
                    let last_year = get_summary_avg(&mut db, 31536000);

                    response_body = serde_json::json!({
                        "last_minute": last_minute,
                        "last_hour": last_hour,
                        "last_day": last_day,
                        "last_week": last_week,
                        "last_month": last_month,
                        "last_year": last_year
                    }).to_string();
                } else {
                    let is_avg = url.contains("average");
                    let (n_seconds, window_seconds, is_single) = if url.contains("last_minute") {
                        if is_avg { (60, 60, true) } else { (60, 1, false) }
                    } else if url.contains("last_hour") {
                        if is_avg { (3600, 3600, true) } else { (3600, 60, false) }
                    } else if url.contains("last_day") {
                        if is_avg { (3600 * 24, 3600 * 24, true) } else { (3600 * 24, 3600, false) }
                    } else if url.contains("last_week") {
                        if is_avg { (3600 * 24 * 7, 3600 * 24 * 7, true) } else { (3600 * 24 * 7, 3600 * 24, false) }
                    } else if url.contains("last_month") {
                        if is_avg { (2592000, 2592000, true) } else { (2592000, 86400, false) }
                    } else if url.contains("last_year") {
                        if is_avg { (31536000, 31536000, true) } else { (31536000, 604800, false) }
                    } else if url.contains("current") {
                        (1, 1, true)
                    } else {
                        (0, 0, false)
                    };

                    if window_seconds > 0 {
                        match db.select_last_n_seconds_average(
                            n_seconds, 
                            TotalData::table_name_static(), 
                            window_seconds
                        ) {
                            Ok(data) => {
                                let mut results = Vec::new();
                                for (ts, sensor) in data {
                                    if let SensorData::Total(tot) = sensor {
                                        results.push(serde_json::json!({
                                            "time": ts.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
                                            "power_watts": tot.total_power_watts,
                                            "period_type": tot.period_type,
                                        }));
                                    }
                                }
                                if is_single {
                                    response_body = results.last().cloned().unwrap_or_else(|| serde_json::json!({})).to_string();
                                } else {
                                    response_body = serde_json::to_string(&results).unwrap_or_default();
                                }
                            }
                            Err(e) => {
                                status = StatusCode(500);
                                response_body = format!("{{\"error\": \"{}\"}}", e);
                            }
                        }
                    } else {
                        status = StatusCode(404);
                        response_body = "{\"error\": \"Not found\"}".to_string();
                    }
                }
            } else {
                status = StatusCode(404);
                response_body = "{\"error\": \"Not found\"}".to_string();
            }

            let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
            let response = Response::from_string(response_body).with_status_code(status).with_header(header);
            let _ = request.respond(response);
        }
    });
}
