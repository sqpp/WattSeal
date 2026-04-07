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

        for mut request in server.incoming_requests() {
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
            let mut response_body = String::new();
            let mut status = StatusCode(200);

            // Match simple endpoints matching app chart behaviors
            if url.starts_with("/api/stats/") {
                // (n_seconds, window_seconds)
                let (n_seconds, window_seconds) = if url.contains("last_minute") {
                    (60, 1) // 60 seconds total, 1 second buckets
                } else if url.contains("last_hour") {
                    (3600, 60) // 1 hour total, 1 minute buckets
                } else if url.contains("last_day") {
                    (3600 * 24, 3600) // 24 hours total, 1 hour buckets
                } else if url.contains("last_week") {
                    (3600 * 24 * 7, 3600 * 24) // 7 days total, 1 day buckets
                } else if url.contains("last_month") {
                    (2592000, 86400) // 30 days total, 1 day buckets
                } else if url.contains("last_year") {
                    (31536000, 604800) // 365 days total, 1 week buckets
                } else {
                    (0, 0)
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
                            response_body = serde_json::to_string(&results).unwrap_or_default();
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
