use crate::{Error, AssetLoader};

#[derive(Default)]
pub struct FileAssetLoader;

impl AssetLoader for FileAssetLoader {
    fn load_asset_data(&self, path: &str) -> Result<Vec<u8>, Error> {
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
}
