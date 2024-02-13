use cope::stats::StatsLogger;

pub struct EspStatsLogger {
    path: String,
}

impl StatsLogger for EspStatsLogger {
    fn new(path: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            path: path.to_owned(),
        })
    }

    fn log(&mut self, data: &str) {
        println!("STATS {} {}", self.path, data);
    }
}
