use crate::{Error, abstraction::NodeConfig};
use std::collections::HashMap;
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

pub type ConfigDeserializerFn = Box<
    dyn Fn(
            &serde_json::Value,
        ) -> Result<Box<dyn NodeConfig + Send + Sync + 'static>, serde_json::Error>
        + Send
        + Sync,
>;

pub struct NodeRegistry {
    config_fns: HashMap<String, ConfigDeserializerFn>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            config_fns: HashMap::new(),
        }
    }

    pub fn register_node_type<C>(&mut self, type_name: &str)
    where
        C: NodeConfig + serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let name = type_name.to_string();
        self.config_fns.insert(
            name.clone(),
            Box::new(move |value: &serde_json::Value| {
                let config: C = serde_json::from_value(value.clone())?;
                Ok(Box::new(config))
            }),
        );
    }

    pub fn get_deserialize_fn(&self, type_name: &str) -> Option<&ConfigDeserializerFn> {
        self.config_fns.get(type_name)
    }
}
