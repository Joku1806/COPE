use byte_unit::Byte;
use std::str::FromStr;

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

#[derive(Debug)]
pub enum TrafficGeneratorType {
    None,
    Greedy,
    Poisson(u32),
    Random(u32),
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
        let mean_str = match parts.get(1) {
            None => None,
            // NOTE: Closing brace has to be removed here
            Some(m) => Some(&m[..m.len() - 1]),
        };
        let mean = match mean_str {
            None => None,
            Some(s) => match Byte::parse_str(s, true) {
                Ok(b) => Some(b.as_u64() as u32),
                Err(_) => return Err(TrafficGeneratorTypeError::InvalidFormat),
            },
        };

        let tg = match (dist_str, mean) {
            ("None", None) => TrafficGeneratorType::None,
            ("Greedy", None) => TrafficGeneratorType::Greedy,
            ("Poisson", Some(m)) => TrafficGeneratorType::Poisson(m),
            ("Random", Some(m)) => TrafficGeneratorType::Random(m),
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
            TrafficGeneratorType::Poisson(m) => write!(f, "Poisson({:#})", m),
            TrafficGeneratorType::Random(m) => write!(f, "Random({:#})", m),
        }
    }
}
