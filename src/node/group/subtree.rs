use crate::{
    Error, GraphNode,
    abstraction::{NodeConfigData, NodeConfig, NodeRegistry}
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Subtree {
    pub file_path: String
}

impl Subtree {
    pub fn new(file_path: &str) -> Self {
        Self { file_path: file_path.to_owned() }
    }
}

impl NodeConfig for Subtree {
    fn to_node(&self, registry: &NodeRegistry) -> Result<GraphNode, Error> {
        let asset_data = registry.load_asset(&self.file_path)?;
        let config: NodeConfigData = serde_json::from_slice(&asset_data)?;
        config.0.to_node(registry)
    }

    fn clone_child_configs(&self) -> Option<Vec<NodeConfigData>> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig> {
        Box::new(Self {
            file_path: self.file_path.clone()
        })
    }
}

