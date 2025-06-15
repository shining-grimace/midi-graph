pub mod defaults;
pub mod registry;

use crate::{Error, GraphNode, abstraction::NodeRegistry};
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

pub trait NodeConfig: Send + 'static {
    fn to_node(&self, registry: &NodeRegistry) -> Result<GraphNode, Error>;
    fn clone_child_configs(&self) -> Option<Vec<NodeConfigData>>;
    fn duplicate(&self) -> Box<dyn NodeConfig>;
}

#[derive(Clone)]
pub struct Config {
    pub root: NodeConfigData,
}

/// Loop range, defined as the inclusive start and exclusive end.
/// These points are specified in frames, not data points.
#[derive(Deserialize, Serialize, Clone)]
pub struct Loop {
    pub start: usize,
    pub end: usize,
}

pub struct NodeConfigData(pub Box<dyn NodeConfig>);

impl Clone for NodeConfigData {
    fn clone(&self) -> Self {
        Self(self.0.duplicate())
    }
}

impl std::fmt::Debug for NodeConfigData {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "[NodeConfigData]")
    }
}

impl<'de> Deserialize<'de> for NodeConfigData {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        todo!("Deserialize NodeConfigData")
    }
}

