use crate::{AudioStreamer, Error};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Stream, StreamConfig};

pub struct BaseMixer {
    source: Box<dyn AudioStreamer + Send + 'static>,
}

impl BaseMixer {
    pub fn from_source(source: Box<dyn AudioStreamer + Send + 'static>) -> Self {
        Self { source }
    }

    pub fn open_stream(self) -> Result<Stream, Error> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(Error::NoDevice)?;
        let required_config = StreamConfig {
            buffer_size: cpal::BufferSize::Fixed(2048),
            channels: 2,
            sample_rate: cpal::SampleRate(48000),
        };
        let mut source = self.source;
        let stream = device.build_output_stream(
            &required_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                source.fill_buffer(data);
            },
            move |err| {
                println!("Stream error: {:?}", err);
            },
            None,
        )?;
        Ok(stream)
    }
}
