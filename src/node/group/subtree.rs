use crate::{
    AssetLoadPayload, AssetLoader, Error, GraphNode,
    abstraction::{ChildConfig, NodeConfig},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum SubtreeData {
    FilePath(String),
    Config(ChildConfig),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Subtree {
    pub source: SubtreeData,
}

impl Subtree {
    pub fn as_path(file_path: &str) -> Self {
        Self {
            source: SubtreeData::FilePath(file_path.to_owned()),
        }
    }

    pub fn as_config(config: ChildConfig) -> Self {
        Self {
            source: SubtreeData::Config(config),
        }
    }
}

impl NodeConfig for Subtree {
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        match &self.source {
            SubtreeData::FilePath(file_path) => match asset_loader.load_asset_data(file_path)? {
                AssetLoadPayload::RawAssetData(raw_data) => {
                    let config: ChildConfig = serde_json::from_slice(&raw_data)?;
                    config.0.to_node(asset_loader)
                }
                AssetLoadPayload::PreparedData(_) => Err(Error::User(
                    "ERROR: Prepared file data is not supported for Subtree.".to_owned(),
                )),
            },
            SubtreeData::Config(config) => config.0.to_node(asset_loader),
        }
    }

    fn clone_child_configs(&self) -> Option<Vec<ChildConfig>> {
        None
    }

    fn asset_source(&self) -> Option<&str> {
        match &self.source {
            SubtreeData::FilePath(path) => Some(path),
            SubtreeData::Config(_) => None,
        }
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        let new_self: Self = match &self.source {
            SubtreeData::FilePath(file_path) => Self::as_path(file_path),
            SubtreeData::Config(config) => Self::as_config(config.clone()),
        };
        Box::new(new_self)
    }
}
