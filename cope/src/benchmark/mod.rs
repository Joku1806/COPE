use std::time::{Duration, Instant};
use std::collections::HashMap;

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
}

impl BenchTimer {
    pub fn new() -> Self {
        Self {
            timer: Instant::now(),
            snapshots: HashMap::new(),
        }
    }
    pub fn reset(&mut self) {
        self.timer = Instant::now();
    }
    pub fn snapshot(&mut self, name: &'static str) {
        let time_elap = self.timer.elapsed();
        if let Some(snap) = self.snapshots.get_mut(name) {
            snap.mesurement_count+=1;
            snap.duration += time_elap;
            return;
        }
        let snap = Snapshot {
            duration: time_elap,
            mesurement_count: 1,
        };
        self.snapshots.insert(name, snap);
    }

    pub fn log(&self, id: NodeID) {
        for (name, snap) in &self.snapshots {
            log::debug!("[Node {}][Benchmark]: {}, {:?}", id, name, snap.avg());
        }
    }
}
