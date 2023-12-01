use crate::packet::Packet;

pub mod greedy_generator;
pub mod none_generator;
pub mod pareto_generator;
pub mod poisson_generator;
pub mod random_generator;

pub trait TrafficGenerator {
    fn generate(&mut self) -> Option<Packet>;
}
