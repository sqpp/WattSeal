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
                    // Summary policy:
                    // - short ranges (minute/hour): prefer completed bucket for stability
                    // - long ranges (day/week/month/year): include current ongoing data
                    let get_summary_avg = |db: &mut Database, period_secs: i64, include_current: bool| -> f64 {
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

                            if include_current {
                                current
                            } else if previous > 0.0 {
                                previous
                            } else {
                                current
                            }
                        } else { 0.0 }
                    };

                    let last_minute = get_summary_avg(&mut db, 60, false);
                    let last_hour = get_summary_avg(&mut db, 3600, false);
                    let last_day = get_summary_avg(&mut db, 3600 * 24, true);
                    let last_week = get_summary_avg(&mut db, 3600 * 24 * 7, true);
                    let last_month = get_summary_avg(&mut db, 2592000, true);
                    let last_year = get_summary_avg(&mut db, 31536000, true);

                    let all_time_wh = db
                        .get_all_time_data()
                        .ok()
                        .and_then(|all| all.components.get(TotalData::table_name_static()).copied())
                        .unwrap_or(0.0)
                        .max(0.0);

                    let settings = db.load_ui_settings().ok().flatten();
                    let carbon_g_per_kwh = settings
                        .as_ref()
                        .map(|s| parse_carbon_intensity_g_per_kwh(&s.carbon_intensity))
                        .unwrap_or(399.0);
                    let usd_per_kwh = settings
                        .as_ref()
                        .map(|s| parse_electricity_cost_usd_per_kwh(&s.kwh_cost))
                        .unwrap_or(0.17);
                    let emissions_g_co2 = (all_time_wh / 1000.0) * carbon_g_per_kwh;
                    let estimated_bill_usd = (all_time_wh / 1000.0) * usd_per_kwh;

                    response_body = serde_json::json!({
                        "last_minute": last_minute,
                        "last_hour": last_hour,
                        "last_day": last_day,
                        "last_week": last_week,
                        "last_month": last_month,
                        "last_year": last_year,
                        "all_time_wh": all_time_wh,
                        "emissions_g_co2": emissions_g_co2,
                        "estimated_bill_usd": estimated_bill_usd
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

fn parse_carbon_intensity_g_per_kwh(raw: &str) -> f64 {
    let label = raw.trim();
    if let Ok(v) = label.parse::<f64>() {
        return v.max(0.0);
    }
    match label {
        "France" => 42.0,
        "Germany" => 332.0,
        "UK" => 217.0,
        "USA (average)" => 384.0,
        "China" => 555.0,
        "India" => 707.0,
        "Sweden" => 35.0,
        "Poland" => 592.0,
        "World average" => 399.0,
        _ => 399.0,
    }
}

fn parse_electricity_cost_usd_per_kwh(raw: &str) -> f64 {
    let label = raw.trim();
    if let Ok(v) = label.parse::<f64>() {
        return v.max(0.0);
    }
    match label {
        "France" => 0.28,
        "Germany" => 0.4,
        "Spain" => 0.25,
        "Italy" => 0.42,
        "Netherlands" => 0.28,
        "Switzerland" => 0.37,
        "UK" => 0.4,
        "USA (average)" => 0.18,
        "World average" => 0.17,
        _ => 0.17,
    }
}
