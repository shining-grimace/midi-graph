use crate::{AssetLoader, Error, abstraction::NodeConfigData};
use std::sync::OnceLock;

static NODE_REGISTRY: OnceLock<NodeRegistry> = OnceLock::new();

pub fn init_node_registry(registry: NodeRegistry) -> Result<(), Error> {
    NODE_REGISTRY.set(registry).map_err(|_| {
        Error::User("Error calling init_node_registry: already initialised".to_owned())
    })
}

pub(crate) fn get_registry() -> Option<&'static NodeRegistry> {
    NODE_REGISTRY.get()
}

pub struct NodeRegistry {
    asset_loader: Box<dyn AssetLoader + Send + Sync>,
}

impl NodeRegistry {
    pub fn new(asset_loader: Box<dyn AssetLoader + Send + Sync + 'static>) -> Self {
        Self { asset_loader }
    }

    pub fn load_asset(&self, path: &str) -> Result<Vec<u8>, Error> {
        self.asset_loader.load_asset_data(path)
    }

    pub fn traverse_config_tree(
        &self,
        config: &NodeConfigData,
        touch_node: &mut dyn FnMut(&NodeConfigData),
    ) -> Result<(), Error> {
        touch_node(config);
        if let Some(children) = config.0.clone_child_configs() {
            for child in children.iter() {
                touch_node(child);
            }
        }
        Ok(())
    }
}
