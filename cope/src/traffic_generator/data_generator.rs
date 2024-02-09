use rand::Rng;
use std::cmp::min;

const SAMPLES: [&str; 3] = [
    "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua.",
    "Die Galaktische Republik wird von Unruhen erschüttert. Die Besteuerung der Handelsrouten zu weit entfernten Sternensystemen ist der Auslöser.",
    "If life seems jolly rotten There's something you've forgotten And that's to laugh and smile and dance and sing",
];

pub struct DataGenerator {}

impl DataGenerator {
    pub fn new() -> Self {
        DataGenerator {}
    }

    pub fn generate(&self, size: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..SAMPLES.len());
        let sample = SAMPLES[index];
        let mut current_size = 0;
        let mut generated = vec![];

        while current_size < size {
            let append_size = min(size - current_size, sample.len());
            current_size += append_size;

            let bytes = sample.as_bytes();
            generated.extend(&bytes[..append_size]);
        }

        generated
    }
}
