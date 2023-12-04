// use crate::packet::{Packet, PacketID};
// use crate::traffic_generator::TGStrategy;

// pub struct GreedyGenerator {
//     current_packet_id: PacketID,
// }

// // NOTE: A generator that will always return a packet.
// // Useful for measuring maximum network throughput.
// impl GreedyGenerator {
//     pub fn new() -> Self {
//         GreedyGenerator {
//             current_packet_id: 0,
//         }
//     }
// }

// impl TGStrategy for GreedyGenerator {
//     fn generate(&mut self) -> Option<Packet> {
//         let mut p = Packet::with_serialized_size(256).unwrap();
//         p.set_id(self.current_packet_id);
//         self.current_packet_id += 1;

//         return Some(p);
//     }
// }
