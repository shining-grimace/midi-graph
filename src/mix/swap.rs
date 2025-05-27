use crate::GraphNode;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};

pub struct SwappableConsumer {
    consumer: Arc<AtomicPtr<GraphNode>>,
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
    pub fn new(consumer: GraphNode) -> Self {
        let boxed_consumer = Box::new(consumer);
        let consumer_arc = Arc::new(AtomicPtr::new(Box::into_raw(boxed_consumer)));
        Self {
            consumer: consumer_arc,
        }
    }

    pub fn take_consumer(&self) -> Arc<AtomicPtr<GraphNode>> {
        Arc::clone(&self.consumer)
    }

    pub fn swap_consumer(
        &mut self,
        consumer: GraphNode,
    ) -> Option<GraphNode> {
        let boxed_consumer = Box::new(consumer);
        let old_ptr = self
            .consumer
            .swap(Box::into_raw(boxed_consumer), Ordering::SeqCst);
        if !old_ptr.is_null() {
            unsafe {
                let boxed_consumer: Box<GraphNode> = Box::from_raw(old_ptr);
                Some(*boxed_consumer)
            }
        } else {
            None
        }
    }
}
