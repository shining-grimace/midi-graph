use crate::{consts, Error, Node, NullSource};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, Mutex,
};

struct SwappableConsumer {
    consumer: Arc<AtomicPtr<Box<dyn Node + Send + 'static>>>,
}

impl Drop for SwappableConsumer {
    fn drop(&mut self) {
        let ptr = self.consumer.load(Ordering::SeqCst);
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }
}

impl SwappableConsumer {
    pub fn new(consumer: Box<dyn Node + Send + 'static>) -> Self {
        let boxed_consumer = Box::new(consumer);
        let consumer_arc = Arc::new(AtomicPtr::new(Box::into_raw(boxed_consumer)));
        Self {
            consumer: consumer_arc,
        }
    }

    pub fn take_consumer(&self) -> Arc<AtomicPtr<Box<dyn Node + Send + 'static>>> {
        Arc::clone(&self.consumer)
    }

    pub fn swap_consumer(&mut self, consumer: Box<dyn Node + Send + 'static>) {
        let boxed_consumer = Box::new(consumer);
        let old_ptr = self
            .consumer
            .swap(Box::into_raw(boxed_consumer), Ordering::SeqCst);
        if !old_ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(old_ptr); // Drop the old consumer
            }
        }
    }
}

pub struct BaseMixer {
    stream: Mutex<Stream>,
    consumer: SwappableConsumer,
}

impl Drop for BaseMixer {
    fn drop(&mut self) {
        let stream = self.stream.lock().expect("Could not lock the audio stream");
        stream.pause().expect("Could not pause the stream");
    }
}

impl BaseMixer {
    pub fn start_empty() -> Result<Self, Error> {
        let consumer = Box::new(NullSource::new(None));
        Self::start_with(consumer)
    }

    pub fn start_with(consumer: Box<dyn Node + Send + 'static>) -> Result<Self, Error> {
        let swappable = SwappableConsumer::new(consumer);
        let stream = Self::open_stream(swappable.take_consumer())?;
        stream.play()?;
        Ok(Self {
            stream: Mutex::new(stream),
            consumer: swappable,
        })
    }

    fn open_stream(
        consumer: Arc<AtomicPtr<Box<dyn Node + Send + 'static>>>,
    ) -> Result<Stream, Error> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(Error::NoDevice)?;
        let required_config = StreamConfig {
            buffer_size: cpal::BufferSize::Fixed(consts::BUFFER_SIZE as u32),
            channels: consts::CHANNEL_COUNT as u16,
            sample_rate: cpal::SampleRate(consts::PLAYBACK_SAMPLE_RATE as u32),
        };
        let stream = device.build_output_stream(
            &required_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                data.fill(0.0);
                let consumer_ptr = consumer.load(Ordering::SeqCst);
                if !consumer_ptr.is_null() {
                    unsafe {
                        (*consumer_ptr).fill_buffer(data);
                    }
                }
            },
            move |err| {
                println!("ERROR: Stream: {:?}", err);
            },
            None,
        )?;
        Ok(stream)
    }

    pub fn swap_consumer(&mut self, consumer: Box<dyn Node + Send + 'static>) {
        self.consumer.swap_consumer(consumer);
    }
}
