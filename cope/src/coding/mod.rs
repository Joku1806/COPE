use super::Packet;
use std::collections::VecDeque;

trait CodingStrategy {}
struct NodeCodingNone {
    _packet_fifo: VecDeque<Packet>,
}
struct RelayCodingNone {
    _packet_fifo: VecDeque<Packet>,
}

impl CodingStrategy for NodeCodingNone {}
impl CodingStrategy for RelayCodingNone {}
