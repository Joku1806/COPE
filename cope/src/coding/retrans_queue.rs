use std::{
    time::{Duration, Instant},
    usize,
};

use crate::{
    config::CONFIG,
    packet::{CodingInfo, PacketData},
};

#[derive(Debug)]
pub struct RetransEntry {
    data: PacketData,
    info: CodingInfo,
    retrans_count: u8,
    last_trans: Instant,
}

#[derive(Debug)]
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

    pub fn conatains(&self, info: &CodingInfo) -> bool {
        self.queue
            .iter()
            .find(|entry| entry.info == *info)
            .is_some()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn packet_to_retrans(&mut self) -> Option<(CodingInfo, PacketData)> {
        let Some(entry_pos) = self.queue.iter().position(|entry| {
            let duration = entry.last_trans.elapsed();
            duration >= self.retrans_duration
        }) else {
            return None;
        };

        let new_instant = Instant::now();
        if self.queue[entry_pos].retrans_count < CONFIG.max_retrans_amount {
            let entry = &mut self.queue[entry_pos];
            entry.last_trans = new_instant;
            entry.retrans_count += 1;
            return Some((entry.info.clone(), entry.data.clone()));
        }
        let mut entry = self.queue.remove(entry_pos);
        entry.last_trans = new_instant;
        entry.retrans_count += 1;
        return Some((entry.info, entry.data));
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
