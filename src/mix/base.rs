use crate::{consts, Error, Node};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Stream, StreamConfig};

pub struct BaseMixer {
    consumer: Box<dyn Node + Send + 'static>,
}

impl BaseMixer {
    pub fn from_consumer(consumer: Box<dyn Node + Send + 'static>) -> Self {
        Self { consumer }
    }

    pub fn open_stream(self) -> Result<Stream, Error> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(Error::NoDevice)?;
        let required_config = StreamConfig {
            buffer_size: cpal::BufferSize::Fixed(consts::BUFFER_SIZE as u32),
            channels: consts::CHANNEL_COUNT as u16,
            sample_rate: cpal::SampleRate(consts::PLAYBACK_SAMPLE_RATE as u32),
        };
        let mut consumer = self.consumer;
        let stream = device.build_output_stream(
            &required_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                data.fill(0.0);
                consumer.fill_buffer(data);
            },
            move |err| {
                println!("ERROR: Stream: {:?}", err);
            },
            None,
        )?;
        Ok(stream)
    }
}
