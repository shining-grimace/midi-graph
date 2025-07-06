use std::sync::atomic::{AtomicUsize, Ordering};

static DEBUG_LOGGING_MODE: AtomicUsize = AtomicUsize::new(0);

pub struct DebugLogging;

impl DebugLogging {
    const LOG_INIT_MASK: usize = 0x0001;

    pub fn get_log_on_init() -> bool {
        (DEBUG_LOGGING_MODE.load(Ordering::Relaxed) & Self::LOG_INIT_MASK) != 0
    }

    pub fn set_logging_on_init() {
        let mode = DEBUG_LOGGING_MODE.load(Ordering::Relaxed);
        DEBUG_LOGGING_MODE.store(mode | Self::LOG_INIT_MASK, Ordering::Relaxed);
    }
}
