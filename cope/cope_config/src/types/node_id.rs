use std::cmp::Eq;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum NodeIDError {
    NotSingleCharacter,
    NotUppercase,
}

impl fmt::Display for NodeIDError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeIDError::NotSingleCharacter => f.write_fmt(format_args!("Not a single character")),
            NodeIDError::NotUppercase => f.write_fmt(format_args!("Not an uppercase character")),
        }
    }
}

// A char that has to be an uppercase letter
// Maybe we change this from char into u8?
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct NodeID(char);
impl NodeID {
    pub const fn new(c: char) -> Self {
        Self(c)
    }
}

impl FromStr for NodeID {
    type Err = NodeIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.len() == 1 {
            return Err(Self::Err::NotSingleCharacter);
        }

        let c: char = s.chars().next().unwrap();
        if !c.is_uppercase() {
            return Err(Self::Err::NotUppercase);
        }

        Ok(Self(c))
    }
}

impl TryFrom<char> for NodeID {
    type Error = NodeIDError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if !value.is_uppercase() {
            return Err(Self::Error::NotUppercase);
        }

        Ok(Self(value))
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
