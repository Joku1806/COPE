use std::time::{Instant, Duration};

use crate::packet::{CodingInfo, PacketData};

pub struct RetransEntry {
    data: PacketData,
    info: CodingInfo,
    retrans_count: u8,
    last_trans: Instant,
}

pub struct RetransQueue {
    queue: Vec<RetransEntry>,
    max_count: usize,
    retrans_duration: Duration,
}

impl RetransQueue {
    pub fn new(max_count: usize, retrans_duration: Duration) -> Self {
        Self {
            queue: vec![],
            max_count,
            retrans_duration,
        }
    }

    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.max_count
    }

    pub fn packet_to_retrans(&mut self) -> Option<(CodingInfo, PacketData)> {
        for entry in self.queue.iter_mut() {
            let duration = entry.last_trans.elapsed();
            if duration >= self.retrans_duration {
                let new_instant = Instant::now();
                entry.last_trans = new_instant;
                entry.retrans_count += 1;
                return Some((entry.info.clone(), entry.data.clone()));
            }
        }

        None
    }

    pub fn push_new(&mut self, packet: (CodingInfo, PacketData)) {
        let instant = Instant::now();
        let entry = RetransEntry {
            data: packet.1,
            info: packet.0,
            retrans_count: 0,
            last_trans: instant,
        };
        self.queue.push(entry);
    }

    pub fn remove_packet(&mut self, info: &CodingInfo) {
        let Some(pos) = self.queue.iter().position(|entry| entry.info == *info) else {
            return;
        };
        self.queue.remove(pos);
    }
}
