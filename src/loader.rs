use crate::Error;
use std::sync::Arc;

pub type SerializedFileMetadata = Arc<Vec<u8>>;

pub type SampleBuffer = Arc<Vec<f32>>;

pub enum AssetLoadPayload {
    RawAssetData(Vec<u8>),
    PreparedData((SerializedFileMetadata, SampleBuffer)),
}

pub trait AssetLoader {
    fn load_asset_data(&mut self, path: &str) -> Result<AssetLoadPayload, Error>;
    fn store_prepared_data(
        &mut self,
        path: &str,
        metadata: SerializedFileMetadata,
        sample_buffer: SampleBuffer,
    );
}
