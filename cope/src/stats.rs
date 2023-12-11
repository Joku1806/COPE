use cope_config::types::node_id::NodeID;
use crate::Packet;
use crate::config::CONFIG;

use std::fs;
use std::io::Write;

pub struct Stats {
    file: fs::File,
    bench_duration: std::time::Duration,
    time_stamp: std::time::Instant,
    packages_send_to: Vec<(NodeID, u32)>,
    packages_rec_from: Vec<(NodeID, u32)>,
    packets_send: u32,
    packets_rec: u32,
    coded_rec: u32,
    decoded_rec: u32,
    report_rec: u32,
    overhearded: u32,
    total_data_send: u32,
    total_data_rec: u32,
}

// NOTE: For the ESP we can just print to std out
// NOTE: We should probably

// Were do I put this function
fn folder_exists(folder_path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(folder_path) {
        return metadata.is_dir();
    }
    false
}

impl Stats {
    pub fn new(node_id: NodeID, duration: std::time::Duration) -> std::io::Result<Self> {
        if !folder_exists("log") {
            fs::create_dir("log")?;
        }
        let path = format!("./log/log_{}.csv", node_id.unwrap());
        let file = fs::File::create(path)?;

        let send_to: Vec<(NodeID, u32)> = CONFIG
            .get_rx_whitelist_for(node_id)
            .expect("Config should contain rx whitelist")
            .iter()
            .map(|id| (*id, 0u32))
            .collect();

        let rec_from: Vec<(NodeID, u32)> = CONFIG
            .get_tx_whitelist_for(node_id)
            .expect("Config should contain tx whitelist")
            .iter()
            .map(|id| (*id, 0u32))
            .collect();

        let mut stats = Self {
            file,
            bench_duration: duration,
            time_stamp: std::time::Instant::now(),
            packages_send_to: send_to,
            packages_rec_from: rec_from,
            packets_send: 0,
            packets_rec: 0,
            coded_rec: 0,
            decoded_rec: 0,
            report_rec: 0,
            overhearded: 0,
            total_data_send: 0,
            total_data_rec: 0
        };

        stats.write_file_header()?;

        Ok(stats)
    }

    pub fn record(&mut self) {
        let  time_elapsed = self.time_stamp.elapsed();
        if time_elapsed > self.bench_duration {
            self.write_to_file().unwrap();
            self.reset();
            self.time_stamp = std::time::Instant::now();
        }
    }

    fn write_file_header(&mut self) -> std::io::Result<()>{
        write!(self.file, "time,")?;
        for (id, _) in &self.packages_send_to {
            write!(self.file, "send_to_{},", id.unwrap())?;
        }
        for (id, _) in &self.packages_rec_from {
            write!(self.file, "rec_from_{},", id.unwrap())?;
        }

        write!(self.file, "packets_send,packets_rec,")?;
        writeln!(self.file, "total_data_send,total_data_rec")?;
        Ok(())
    }

    pub fn write_to_file(&mut self) -> std::io::Result<()> {
        write!(self.file, "{},", self.time_stamp.elapsed().as_secs())?;
        for (_, val) in &self.packages_send_to {
            write!(self.file, "{},", val)?;
        }
        for (_, val) in &self.packages_rec_from {
            write!(self.file, "{},", val)?;
        }

        write!(self.file, "{},{},", self.packets_send, self.packets_rec)?;
        writeln!(self.file, "{},{}", self.total_data_send, self.total_data_rec)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.time_stamp = std::time::Instant::now();
        for (_, val) in &mut self.packages_send_to {
            *val = 0;
        }
        for (_, val) in &mut self.packages_rec_from {
            *val = 0;
        }
        self.packets_rec = 0;
        self.packets_send = 0;
        self.total_data_send = 0;
        self.total_data_rec = 0;
    }

    pub fn add_send(&mut self, packet: &Packet){
        self.packets_send += 1;
        self.total_data_send += packet.data().size() as u32;
    }

    pub fn add_rec(&mut self, packet: &Packet){
        self.packets_rec += 1;
        self.total_data_rec += packet.data().size() as u32;
        let header_len = packet.coding_header().len();
        match header_len {
            0 => self.report_rec += 1,
            1 => (),
            _ => self.coded_rec += 1,
        }
    }

    pub fn add_decoded(&mut self){
        self.decoded_rec += 1;
    }

    pub fn add_overheard(&mut self){
        self.overhearded += 1;
    }
}

