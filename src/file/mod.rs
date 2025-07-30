use crate::{AssetLoadPayload, AssetLoader, Error, SampleBuffer, SerializedFileMetadata};
use std::collections::HashMap;

#[derive(Default)]
pub struct FileAssetLoader {
    loaded_files: HashMap<String, (SerializedFileMetadata, SampleBuffer)>,
}

impl AssetLoader for FileAssetLoader {
    fn load_asset_data(&mut self, path: &str) -> Result<AssetLoadPayload, Error> {
        let path = path.to_owned();
        if let Some((metadata, sample_buffer)) = self.loaded_files.get(&path) {
            return Ok(AssetLoadPayload::PreparedData((
                metadata.clone(),
                sample_buffer.clone(),
            )));
        };
        let bytes = std::fs::read(path)?;
        Ok(AssetLoadPayload::RawAssetData(bytes))
    }

    fn store_prepared_data(
        &mut self,
        path: &str,
        metadata: SerializedFileMetadata,
        sample_buffer: SampleBuffer,
    ) {
        self.loaded_files
            .insert(path.to_owned(), (metadata, sample_buffer));
    }
}
