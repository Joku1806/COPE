use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use cope_config::types::node_id::NodeID;

pub struct Snapshot {
    pub duration: Duration,
    pub mesurement_count: u32,
}

impl Snapshot {
    fn avg(&self) -> Duration {
        self.duration / self.mesurement_count
    }
}

pub struct BenchTimer {
    timer: Instant,
    snapshots: HashMap<&'static str, Snapshot>,
    log_timer: Instant,
    log_sleep_duration: Duration,
    log_file: Option<File>,
    logging_enabled: bool,
}

impl BenchTimer {
    pub fn new() -> Self {
        Self {
            timer: Instant::now(),
            snapshots: HashMap::new(),
            log_timer: Instant::now(),
            log_sleep_duration: Duration::from_secs(1),
            log_file: None,
            logging_enabled: true,
        }
    }

    pub fn set_bench_log_path(&mut self, path: &String) {
        let p = Path::new(path);
        if let Some(dirs) = p.parent() {
            std::fs::create_dir_all(dirs).unwrap();
        }
        let mut file = OpenOptions::new()
            .append(false)
            .create(true)
            .write(true)
            .open(path)
            .unwrap();
        writeln!(file, "name, avg_time");
        self.log_file = Some(file);
    }

    pub fn reset(&mut self) {
        self.timer = Instant::now();
    }
    pub fn snapshot(&mut self, name: &'static str) {
        let time_elap = self.timer.elapsed();
        if let Some(snap) = self.snapshots.get_mut(name) {
            snap.mesurement_count += 1;
            snap.duration += time_elap;
            return;
        }
        let snap = Snapshot {
            duration: time_elap,
            mesurement_count: 1,
        };
        self.snapshots.insert(name, snap);
    }

    pub fn log(&mut self, id: NodeID) {
        if !self.logging_enabled {
            return;
        }
        if self.log_timer.elapsed() < self.log_sleep_duration {
            return;
        }
        for (name, snap) in &self.snapshots {
            log::debug!("[Node {}][Benchmark]: {}, {:?}", id, name, snap.avg());
            let Some(ref mut file) = self.log_file else {
                continue;
            };
            writeln!(file, "{}, {:?}", name, snap.avg());
        }
        self.log_timer = Instant::now();
    }
}
