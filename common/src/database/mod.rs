use std::time::SystemTime;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result, ToSql, params};

use crate::types::{CPUData, GPUData, SensorData};

pub static DATABASE_PATH: &str = "power_monitoring.db";

pub struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open(DATABASE_PATH)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "OFF")?;
        Ok(Database { conn })
    }

    pub fn create_tables_if_not_exists(&mut self, tables: &Vec<impl DatabaseTable>) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS timestamp (
                  id            INTEGER PRIMARY KEY,
                  timestamp     TEXT NOT NULL
                  )",
            [],
        )?;

        for table in tables {
            let create_table_sql = format!(
                "CREATE TABLE IF NOT EXISTS {} ({});",
                table.table_name(),
                table.columns().join(", ")
            );
            tx.execute(&create_table_sql, [])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_event(&mut self, event: &Event) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO timestamp (timestamp) VALUES (?1)",
            params![DateTime::<Utc>::from(event.time()).to_rfc3339()],
        )?;
        let timestamp_id = tx.last_insert_rowid();
        for sensor_data in event.data() {
            let insert_sql = sensor_data.insert_sql();
            let params = sensor_data.insert_params(&timestamp_id);
            tx.execute(&insert_sql, params.as_slice())?;
        }
        tx.commit()?;
        Ok(())
    }
}

pub trait DatabaseTable {
    fn table_name(&self) -> &'static str;
    fn columns(&self) -> &'static [&'static str];
}

pub trait DatabaseEntry {
    fn table_name_static() -> &'static str;
    fn insert_sql(&self) -> String;
    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql>;
    fn select_last_n_sql(&self, n: i64) -> String;
}

impl DatabaseEntry for SensorData {
    fn table_name_static() -> &'static str {
        "sensor_data"
    }

    fn insert_sql(&self) -> String {
        match self {
            SensorData::CPU(data) => data.insert_sql(),
            SensorData::GPU(data) => data.insert_sql(),
            _ => "".to_string(),
        }
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        match self {
            SensorData::CPU(data) => data.insert_params(timestamp_id),
            SensorData::GPU(data) => data.insert_params(timestamp_id),
            _ => vec![],
        }
    }

    fn select_last_n_sql(&self, n: i64) -> String {
        let name = match self {
            SensorData::CPU(_) => CPUData::table_name_static(),
            SensorData::GPU(_) => GPUData::table_name_static(),
            _ => "",
        };

        format!(
            "SELECT * FROM timestamp JOIN {} ON timestamp.id = {}.timestamp_id ORDER BY timestamp.id DESC LIMIT {}",
            name, name, n
        )
    }
}

impl DatabaseEntry for CPUData {
    fn table_name_static() -> &'static str {
        "cpu_data"
    }

    fn insert_sql(&self) -> String {
        format!("INSERT INTO {} (timestamp_id, total_power_watts, pp0_power_watts, pp1_power_watts, dram_power_watts, usage_percent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", Self::table_name_static()).to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_power_watts,
            &self.pp0_power_watts,
            &self.pp1_power_watts,
            &self.dram_power_watts,
            &self.usage_percent,
        ]
    }

    fn select_last_n_sql(&self, _n: i64) -> String {
        "".to_string()
    }
}

impl DatabaseEntry for GPUData {
    fn table_name_static() -> &'static str {
        "gpu_data"
    }

    fn insert_sql(&self) -> String {
        format!("INSERT INTO {} (timestamp_id, total_power_watts, usage_percent, vram_usage_percent) VALUES (?1, ?2, ?3, ?4)", Self::table_name_static()).to_string()
    }

    fn insert_params<'a>(&'a self, timestamp_id: &'a i64) -> Vec<&'a dyn ToSql> {
        vec![
            timestamp_id,
            &self.total_power_watts,
            &self.usage_percent,
            &self.vram_usage_percent,
        ]
    }

    fn select_last_n_sql(&self, _n: i64) -> String {
        "".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    time: SystemTime,
    data: Vec<SensorData>,
}

impl Event {
    pub fn new(time: SystemTime, data: Vec<SensorData>) -> Self {
        Event { time, data }
    }

    pub fn time(&self) -> SystemTime {
        self.time
    }

    pub fn data(&self) -> &Vec<SensorData> {
        &self.data
    }
}
