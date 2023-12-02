use std::cmp::Eq;
use std::fmt::Display;
use std::ops::Deref;

// A char that has to be an uppercase letter
// Maybe we change this from char into u8?
#[derive(Copy, Clone, Debug)]
pub struct NodeID(char);
impl NodeID {
    pub fn new(value: char) -> Self {
        if value.is_uppercase() {
            panic!("Cannot assign a non Uppercase Value to NodeID");
        }
        Self(value)
    }

    // maybe I can ensure typesafety with macro rules?
    // but a topic for another time
    pub const fn cnst(value: char) -> Self {
        // if value.is_uppercase() {
        //     panic!("Cannot assign a non Uppercase Value to NodeID");
        // }
        Self(value)
    }

    pub fn from_string(str: &String) -> Self {
        assert_eq!(1, str.len());
        let c: char = str.chars().next().unwrap();
        assert_eq!(true, c.is_uppercase());
        Self(c)
    }
}

impl Deref for NodeID {
    type Target = char;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for NodeID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for NodeID {}

impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct MacAdress(pub [u8; 6]);
impl MacAdress {
    pub fn new(array: [u8; 6]) -> Self {
        Self(array)
    }

    pub const fn cnst(array: [u8; 6]) -> Self {
        Self(array)
    }

    pub fn from_string(str: &String) -> Self {
        assert_eq!(12, str.len());
        //check for hexadigit
        // TODO: translate 12 hexadigits into 6 8bit sequence
        Self([0, 0, 0, 0, 0, 0])
    }
}

impl IntoIterator for &MacAdress {
    type Item = u8;
    type IntoIter = std::array::IntoIter<u8, 6>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

trait CopeConfig {}

#[derive(Debug)]
pub struct TmpConfig {
    node_count: usize,
    nodes: Vec<(NodeID, MacAdress)>,
    relay: NodeID,
    black_list: Vec<(NodeID, Vec<NodeID>)>,
}

impl TmpConfig {
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

    pub fn node_count(&self) -> usize {
        self.node_count
    }
    pub fn nodes(&self) -> &Vec<(NodeID, MacAdress)> {
        &self.nodes
    }
    pub fn relay(&self) -> NodeID {
        self.relay
    }
    pub fn black_list(&self) -> &Vec<(NodeID, Vec<NodeID>)> {
        &self.black_list
    }
}

impl CopeConfig for TmpConfig {}

pub struct Config<const N: usize> {
    pub nodes: [(NodeID, MacAdress); N],
    pub relay: NodeID,
    // we technically only need N-1 nodes here but yeah
    pub black_list: [(NodeID, [Option<NodeID>; N]); N],
}

impl<const N: usize> CopeConfig for Config<N> {}
