use crate::BufferConsumerNode;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};

pub struct SwappableConsumer {
    consumer: Arc<AtomicPtr<Box<dyn BufferConsumerNode + Send + 'static>>>,
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
    pub fn new(consumer: Box<dyn BufferConsumerNode + Send + 'static>) -> Self {
        let boxed_consumer = Box::new(consumer);
        let consumer_arc = Arc::new(AtomicPtr::new(Box::into_raw(boxed_consumer)));
        Self {
            consumer: consumer_arc,
        }
    }

    pub fn take_consumer(&self) -> Arc<AtomicPtr<Box<dyn BufferConsumerNode + Send + 'static>>> {
        Arc::clone(&self.consumer)
    }

    pub fn swap_consumer(&mut self, consumer: Box<dyn BufferConsumerNode + Send + 'static>) {
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
