use byte_unit::Byte;
use parse_duration;
use std::str::FromStr;
use std::time::Duration;

pub enum TrafficGeneratorTypeError {
    MissingMean,
    InvalidMean,
    InvalidFormat,
    // NOTE: Stattdessen UnknownGenerator?
    InvalidDistribution,
}

impl std::fmt::Display for TrafficGeneratorTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        type Error = TrafficGeneratorTypeError;

        match self {
            Error::MissingMean => f.write_fmt(format_args!("No distribution mean supplied")),
            Error::InvalidMean => f.write_fmt(format_args!(
                "Distribution mean supplied for traffic generator without distribution"
            )),
            Error::InvalidFormat => f.write_fmt(format_args!("Invalid format")),
            Error::InvalidDistribution => f.write_fmt(format_args!("Invalid distribution")),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TrafficGeneratorType {
    None,
    Greedy,
    Poisson(u32),
    Random(u32),
    Periodic(Duration),
}

impl TrafficGeneratorType {
    fn parse_byte_argument(s: &str) -> Result<u32, TrafficGeneratorTypeError> {
        match Byte::parse_str(s, true) {
            Ok(b) => Ok(b.as_u64() as u32),
            Err(_) => return Err(TrafficGeneratorTypeError::InvalidFormat),
        }
    }

    fn parse_duration_argument(s: &str) -> Result<Duration, TrafficGeneratorTypeError> {
        match parse_duration::parse(s) {
            Ok(d) => Ok(d),
            Err(_) => return Err(TrafficGeneratorTypeError::InvalidFormat),
        }
    }
}

impl FromStr for TrafficGeneratorType {
    type Err = TrafficGeneratorTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE: This entire thing could be done more robustly by using regex.
        // I have done it this way for now, because I am not that good at (Rust) regex.
        let parts: Vec<&str> = s.split("(").collect();

        if parts.is_empty() || parts.len() > 2 {
            return Err(TrafficGeneratorTypeError::InvalidFormat);
        }

        let dist_str = parts[0];
        // NOTE: We need to drop the closing brace here
        let arg = parts.get(1).map(|arg| &arg[..arg.len() - 1]);

        let tg = match (dist_str, arg) {
            ("None", None) => TrafficGeneratorType::None,
            ("Greedy", None) => TrafficGeneratorType::Greedy,
            ("Poisson", Some(m)) => {
                TrafficGeneratorType::Poisson(TrafficGeneratorType::parse_byte_argument(m)?)
            }
            ("Random", Some(m)) => {
                TrafficGeneratorType::Random(TrafficGeneratorType::parse_byte_argument(m)?)
            }
            ("Periodic", Some(d)) => {
                TrafficGeneratorType::Periodic(TrafficGeneratorType::parse_duration_argument(d)?)
            }
            ("None", Some(_)) => return Err(TrafficGeneratorTypeError::InvalidMean),
            ("Greedy", Some(_)) => return Err(TrafficGeneratorTypeError::InvalidMean),
            ("Poisson", None) => return Err(TrafficGeneratorTypeError::MissingMean),
            ("Random", None) => return Err(TrafficGeneratorTypeError::MissingMean),
            (_, _) => return Err(TrafficGeneratorTypeError::InvalidDistribution),
        };

        Ok(tg)
    }
}

impl std::fmt::Display for TrafficGeneratorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrafficGeneratorType::None => write!(f, "None"),
            TrafficGeneratorType::Greedy => write!(f, "Greedy"),
            // NOTE: Is it a problem if we lose precision here?
            TrafficGeneratorType::Poisson(m) => write!(f, "Poisson({})", m),
            TrafficGeneratorType::Random(m) => write!(f, "Random({})", m),
            TrafficGeneratorType::Periodic(p) => write!(f, "Periodic({:?})", p),
        }
    }
}
