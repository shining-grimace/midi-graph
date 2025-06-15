use crate::{AssetLoader, Error, abstraction::NodeConfigData};

#[derive(Default)]
pub struct FileAssetLoader;

impl FileAssetLoader {
    pub fn config_from_file(&self, _file_name: &str) -> Result<NodeConfigData, Error> {
        todo!("Deserialize config from a file")
    }
}

impl AssetLoader for FileAssetLoader {
    fn load_asset_data(&self, path: &str) -> Result<Vec<u8>, Error> {
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
}
