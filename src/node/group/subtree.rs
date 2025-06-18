use crate::{
    AssetLoader, Error, GraphNode,
    abstraction::{NodeConfigData, NodeConfig}
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
    fn to_node(&self, asset_loader: &Box<dyn AssetLoader>) -> Result<GraphNode, Error> {
        match &self.source {
            SubtreeData::FilePath(file_path) => {
                let asset_data = asset_loader.load_asset_data(file_path)?;
                let config: NodeConfigData = serde_json::from_slice(&asset_data)?;
                config.0.to_node(asset_loader)
            }
            SubtreeData::Config(config) => config.0.to_node(asset_loader)
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

