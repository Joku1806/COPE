use rand::Rng;
use rand_distr::{Distribution, Normal};

// NOTE: By default, both distributions are weighted equally
const ALPHA: f64 = 0.5;
// NOTE: Kostas Pentikousis and Hussein Badr measured TCP traffic in their paper
// "Quantifying the deployment of TCP options - a comparative study".
// They found the packet size distribution to be bimodally distributed with peaks
// for small packets (<100 Bytes) and large packets (>1400 Bytes).
// We do not send TCP and I eyeballed the stddev values,
// but this should be close enough for our purposes.
const PEAK_SMALL: (f64, f64) = (60.0, 20.0);
const PEAK_LARGE: (f64, f64) = (1400.0, 100.0);

pub struct SizeDistribution {
    alpha: f64,
    dist_one: Normal<f64>,
    dist_two: Normal<f64>,
}

impl SizeDistribution {
    pub fn new() -> Self {
        SizeDistribution {
            alpha: ALPHA,
            dist_one: Normal::new(PEAK_SMALL.0, PEAK_SMALL.1).unwrap(),
            dist_two: Normal::new(PEAK_LARGE.0, PEAK_LARGE.1).unwrap(),
        }
    }

    // FIXME: Pass rng as a parameter
    pub fn sample<R>(&self, rng: &mut R) -> usize
    where
        R: Rng + ?Sized,
    {
        let size = match rng.gen_bool(self.alpha) {
            true => self.dist_one.sample(rng),
            false => self.dist_two.sample(rng),
        };

        size.clamp(0.0, f64::MAX).round() as usize
    }
}
