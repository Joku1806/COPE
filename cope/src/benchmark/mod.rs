use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use cope_config::types::node_id::NodeID;

pub struct Snapshot {
    pub lats_dur: Duration,
    pub total_dur: Duration,
    pub mesurement_count: u32,
}

impl Snapshot {
    fn avg(&self) -> Duration {
        self.total_dur / self.mesurement_count
    }
}

pub struct BenchTimer {
    snapshots: HashMap<&'static str, Snapshot>,
    recordings: HashMap<&'static str, Instant>,
    log_timer: Instant,
    log_sleep_duration: Duration,
    log_file: Option<File>,
    logging_enabled: bool,
}

impl BenchTimer {
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
            recordings: HashMap::new(),
            log_timer: Instant::now(),
            log_sleep_duration: Duration::from_secs(1),
            log_file: None,
            logging_enabled: true,
        }
    }

    pub fn bench_log_path(&mut self, path: &String) {
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
        writeln!(file, "name, last_time, avg_time");
        self.log_file = Some(file);
    }

    pub fn record(&mut self, name: &'static str) {
        self.recordings.insert(name, Instant::now());
    }

    pub fn stop(&mut self, name: &'static str) {
        let Some(instant) = self.recordings.get(name) else {
            log::error!("Missing recording {}!", name);
            return;
        };
        let time_elap = instant.elapsed();
        if let Some(snap) = self.snapshots.get_mut(name) {
            snap.mesurement_count += 1;
            snap.total_dur += time_elap;
            snap.lats_dur = time_elap;
            return;
        }
        let snap = Snapshot {
            lats_dur: time_elap,
            total_dur: time_elap,
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
            log::debug!(
                "[Node {}][Benchmark]: {} => last: {:?}, avg: {:?}.",
                id,
                name,
                snap.lats_dur,
                snap.avg(),
            );
            let Some(ref mut file) = self.log_file else {
                continue;
            };
            writeln!(
                file,
                "{},{},{}",
                name,
                snap.lats_dur.as_nanos(),
                snap.avg().as_nanos()
            );
        }
        self.log_timer = Instant::now();
    }
}
