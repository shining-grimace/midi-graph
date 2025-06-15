use crate::Error;

pub trait AssetLoader {
    fn load_asset_data(&self, path: &str) -> Result<Vec<u8>, Error>;
}
