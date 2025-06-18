use crate::{
    Error, GraphNode,
    abstraction::{NodeConfigData, NodeConfig, NodeRegistry}
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum SubtreeData {
    FilePath(String),
    Config(NodeConfigData)
}

#[derive(Debug, Deserialize, Clone)]
pub struct Subtree {
    pub source: SubtreeData
}

impl Subtree {
    pub fn as_path(file_path: &str) -> Self {
        Self {
            source: SubtreeData::FilePath(file_path.to_owned())
        }
    }

    pub fn as_config(config: NodeConfigData) -> Self {
        Self {
            source: SubtreeData::Config(config)
        }
    }
}

impl NodeConfig for Subtree {
    fn to_node(&self, registry: &NodeRegistry) -> Result<GraphNode, Error> {
        match &self.source {
            SubtreeData::FilePath(file_path) => {
                let asset_data = registry.load_asset(file_path)?;
                let config: NodeConfigData = serde_json::from_slice(&asset_data)?;
                config.0.to_node(registry)
            }
            SubtreeData::Config(config) => config.0.to_node(registry)
        }
    }

    fn clone_child_configs(&self) -> Option<Vec<NodeConfigData>> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        let new_self: Self = match &self.source {
            SubtreeData::FilePath(file_path) => Self::as_path(file_path),
            SubtreeData::Config(config) => Self::as_config(config.clone())
        };
        Box::new(new_self)
    }
}

