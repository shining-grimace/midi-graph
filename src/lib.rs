
// Test suite for the Web and headless browsers.
#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod wasm_tests;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
mod wasm_demo;

use cpal::{Sample, Stream, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait};
use midly::Smf;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Midly(midly::Error),
    Cpal(cpal::BuildStreamError),
    NoDevice
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<midly::Error> for Error {
    fn from(value: midly::Error) -> Self {
        Error::Midly(value)
    }
}

impl From<cpal::BuildStreamError> for Error {
    fn from(value: cpal::BuildStreamError) -> Self {
        Error::Cpal(value)
    }
}

pub trait AudioStreamer {
    fn fill_buffer(&mut self, buffer: &mut [f32]);
}

pub struct SquareAudio {
    period_time: usize,
    is_high: bool
}

impl Default for SquareAudio {
    fn default() -> Self {
        Self { period_time: 0, is_high: false }
    }
}

impl AudioStreamer for SquareAudio {
    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let length = buffer.len();
        for i in 0..length {
            self.period_time += 1;
            if self.period_time >= 32 { // Like 400 Hz at 48 kHz and 2 channels
                self.is_high = !self.is_high;
            }
            buffer[i] = match self.is_high {
                true => Sample::from_sample(0.5),
                false => Sample::from_sample(-0.5)
            };
        }
    }
}

pub struct MidiProcessor {
    smf: Smf<'static>
}

impl MidiProcessor {

    pub fn from_file(file_name: &str) -> Result<MidiProcessor, Error> {
        let bytes = std::fs::read(file_name)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<MidiProcessor, Error> {
        let smf = Smf::parse(&bytes)?.to_static();
        Ok(MidiProcessor {
            smf
        })
    }

    pub fn open_stream<S>(self, mut streamer: S) -> Result<Stream, Error> where S: AudioStreamer + Send + 'static {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(Error::NoDevice)?;
        let required_config = StreamConfig {
            buffer_size: cpal::BufferSize::Fixed(2048),
            channels: 2,
            sample_rate: cpal::SampleRate(48000)
        };
        let stream = device.build_output_stream(
            &required_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                streamer.fill_buffer(data);
            },
            move |err| {
                println!("Stream error: {:?}", err);
            },
            None
        )?;
        Ok(stream)
    }
}
