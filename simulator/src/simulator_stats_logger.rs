use cope::stats::StatsLogger;
use std::fs::OpenOptions;
use std::io::{LineWriter, Write};
use std::path::Path;

pub struct SimulatorStatsLogger {
    line_buffer: LineWriter<std::fs::File>,
}

impl StatsLogger for SimulatorStatsLogger {
    fn new(path: &str) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let p = Path::new(path);

        if let Some(dirs) = p.parent() {
            std::fs::create_dir_all(dirs)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)?;
        let line_buffer = LineWriter::new(file);

        Ok(Self { line_buffer })
    }

    fn log(&mut self, data: &str) {
        log::info!("Logging {} to {:?}", data, self.line_buffer);

        if let Err(e) = writeln!(self.line_buffer, "{}", data) {
            log::warn!("Could not log or flush data: {}", e);
        }
    }
}
