use std::fmt::Display;



// A char that has to be an uppercase letter
#[derive(Copy, Clone,Debug)]
pub struct NodeID(char);
impl NodeID {
    pub fn new(value: char) -> Self {
        if value.is_uppercase() {
            panic!("Cannot assign a non Uppercase Value to NodeID");
        }
        Self(value)
    }
    pub fn from_string(str: &String) -> Self {
        assert_eq!(1, str.len());
        let c: char = str.chars().next().unwrap();
        assert_eq!(true, c.is_uppercase());
        Self(c)
    }
}

impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug)]
pub struct MacAdress([u8; 6]);
impl MacAdress {
    pub fn new(array: [u8; 6]) -> Self {
        Self(array)
    }

    pub fn from_string(str: &String) -> Self {
        assert_eq!(12, str.len());
        //check for hexadigit
        //translate 12 hexadigits into 6 8bit sequence
        // unimplemented!();
        Self([0,0,0,0,0,0])
    }
}

impl IntoIterator for &MacAdress{
    type Item = u8;
    type IntoIter = std::array::IntoIter<u8, 6>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug)]
pub struct Config {
    node_count: usize,
    nodes: Vec<(NodeID, MacAdress)>,
    relay: NodeID,
    black_list: Vec<(NodeID, Vec<NodeID>)>,
}

// impl Default for Config {
// }

impl Config {
    pub fn new(
        nodes: Vec<(NodeID, MacAdress)>,
        relay: NodeID,
        black_list: Vec<(NodeID, Vec<NodeID>)>,
    ) -> Self {
        Self {
            node_count: nodes.len(),
            nodes,
            relay,
            black_list,
        }
    }

    pub fn node_count(&self) -> usize { self.node_count }
    pub fn nodes(&self) -> &Vec<(NodeID, MacAdress)> { &self.nodes }
    pub fn relay(&self) -> NodeID { self.relay }
}
