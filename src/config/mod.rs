pub mod builtin;
pub mod defaults;
pub mod registry;

use crate::{
    AssetLoader, Error, GraphNode, abstraction::NodeRegistry, config::registry::get_registry,
};
use serde::{Deserialize, Serialize, de};
use serde_json::Value;
use std::fmt::Formatter;

pub trait NodeConfig: Send + 'static {
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error>;
    fn clone_child_configs(&self) -> Option<Vec<ChildConfig>>;
    fn asset_source(&self) -> Option<&str>;
    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static>;
}

/// Loop range, defined as the inclusive start and exclusive end.
/// These points are specified in frames, not data points.
#[derive(Deserialize, Serialize, Clone)]
pub struct Loop {
    pub start: usize,
    pub end: usize,
}

pub struct ChildConfig(pub Box<dyn NodeConfig + Send + Sync + 'static>);

impl ChildConfig {
    pub fn traverse_config_tree(config: &Self, touch_node: &mut dyn FnMut(&Self)) {
        touch_node(config);
        if let Some(children) = config.0.clone_child_configs() {
            for child in children.iter() {
                touch_node(child);
            }
        }
    }
}

impl Clone for ChildConfig {
    fn clone(&self) -> Self {
        Self(self.0.duplicate())
    }
}

impl std::fmt::Debug for ChildConfig {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "[ChildConfig]")
    }
}

impl<'de> Deserialize<'de> for ChildConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let full_value = Value::deserialize(deserializer)?;
        let obj = full_value.as_object().ok_or_else(|| {
            de::Error::custom("Error deserializing ChildConfig: not a JSON object")
        })?;
        let node_type = obj.get("type").ok_or_else(|| {
            de::Error::custom("Error deserializing ChildConfig: type key not found")
        })?;
        let node_type_string = node_type.as_str().ok_or_else(|| {
            de::Error::custom("Error deserializing ChildConfig: type is not a string")
        })?;
        let registry = get_registry().ok_or_else(|| {
            de::Error::custom("Error deserializing ChildConfig: start building a BaseMixer first")
        })?;
        let deserializer = registry
            .get_deserialize_fn(node_type_string)
            .ok_or_else(|| {
                de::Error::custom(format!(
                    "Error deserializing ChildConfig: type {} not registered",
                    node_type_string
                ))
            })?;
        let config_trait_object = deserializer(&full_value).map_err(|e| {
            de::Error::custom(format!(
                "Error deserializing ChildConfig: could not deserialize {} node: {}",
                node_type_string, e
            ))
        })?;
        Ok(ChildConfig(config_trait_object))
    }
}
