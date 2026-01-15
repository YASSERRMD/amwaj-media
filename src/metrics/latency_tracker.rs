//! Latency Tracker for component-level timing

use std::time::Instant;

/// Tracks latency for a specific component
/// Automatically records duration when dropped
pub struct LatencyTracker {
    start_time: Instant,
    component: String,
    recorded: bool,
}

impl LatencyTracker {
    /// Create a new latency tracker for a component
    pub fn new(component: &str) -> Self {
        Self {
            start_time: Instant::now(),
            component: component.to_string(),
            recorded: false,
        }
    }

    /// Get the elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64() * 1000.0
    }

    /// Get the component name
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Manually record the latency and return it
    pub fn record(&mut self) -> f64 {
        self.recorded = true;
        self.elapsed_ms()
    }

    /// Check if already recorded
    pub fn is_recorded(&self) -> bool {
        self.recorded
    }

    /// Record with a custom metrics instance
    pub fn record_to(mut self, metrics: &crate::metrics::Metrics) -> f64 {
        let elapsed = self.elapsed_ms();
        metrics.record_latency(elapsed);
        self.recorded = true;
        elapsed
    }
}

/// Scope guard for automatic timing
pub struct ScopedTimer {
    tracker: LatencyTracker,
    callback: Option<Box<dyn FnOnce(f64) + Send>>,
}

impl ScopedTimer {
    /// Create a new scoped timer
    pub fn new<F>(component: &str, callback: F) -> Self
    where
        F: FnOnce(f64) + Send + 'static,
    {
        Self {
            tracker: LatencyTracker::new(component),
            callback: Some(Box::new(callback)),
        }
    }

    /// Create without callback
    pub fn simple(component: &str) -> Self {
        Self {
            tracker: LatencyTracker::new(component),
            callback: None,
        }
    }

    /// Get elapsed time without consuming
    pub fn elapsed_ms(&self) -> f64 {
        self.tracker.elapsed_ms()
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let elapsed = self.tracker.elapsed_ms();
        if let Some(callback) = self.callback.take() {
            callback(elapsed);
        }
    }
}

/// Measure the execution time of a block
#[macro_export]
macro_rules! time_block {
    ($name:expr, $block:block) => {{
        let _start = std::time::Instant::now();
        let result = $block;
        let elapsed = _start.elapsed().as_secs_f64() * 1000.0;
        tracing::debug!("{} took {:.2}ms", $name, elapsed);
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_latency_tracker_creation() {
        let tracker = LatencyTracker::new("test_component");
        assert_eq!(tracker.component(), "test_component");
        assert!(!tracker.is_recorded());
    }

    #[test]
    fn test_latency_tracker_timing() {
        let tracker = LatencyTracker::new("test");
        
        // Wait a bit
        sleep(Duration::from_millis(10));
        
        let elapsed = tracker.elapsed_ms();
        assert!(elapsed >= 9.0); // Allow some tolerance
    }

    #[test]
    fn test_latency_tracker_record() {
        let mut tracker = LatencyTracker::new("test");
        
        sleep(Duration::from_millis(5));
        
        let recorded = tracker.record();
        assert!(recorded >= 4.0);
        assert!(tracker.is_recorded());
    }

    #[test]
    fn test_scoped_timer() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);
        
        {
            let _timer = ScopedTimer::new("test", move |_elapsed| {
                called_clone.store(true, Ordering::SeqCst);
            });
            
            sleep(Duration::from_millis(5));
        }
        
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_scoped_timer_simple() {
        let timer = ScopedTimer::simple("test");
        sleep(Duration::from_millis(5));
        
        assert!(timer.elapsed_ms() >= 4.0);
    }
}
