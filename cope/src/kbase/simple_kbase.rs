use std::collections::HashMap;
use super::KBase;
use crate::packet::CodingInfo;
use cope_config::types::node_id::NodeID;

pub struct SimpleKBase {
    table: HashMap<NodeID, Vec<CodingInfo>>,
    max_size: usize,
}

impl SimpleKBase {
    pub fn new(next_hops: Vec<NodeID>, max_size: usize) -> Self{
        let table = next_hops.iter().map(|&i| (i, vec![])).collect();
        Self { table, max_size }
    }
}

impl KBase for SimpleKBase {
    fn knows(&self, next_hop: &NodeID, info: &CodingInfo) -> bool{
        self.table.get(next_hop)
            .expect("knowledge_base should have a filed for every node!")
            .contains(info)
    }

    fn insert(&mut self, next_hop: NodeID, info: CodingInfo){
        let list = self.table.get_mut(&next_hop)
            .expect("KnowledgeBase should contain Entry for nexthop");
        let is_at_max_size = list.len() >= self.max_size;
        if  is_at_max_size { list.remove(0); }
        list.push(info);
    }

    fn size(&self) -> usize{
        self.table.iter().map(|(_, list)| list.len()).sum()
    }
}
