use std::ops::Sub;
use ordered_float::OrderedFloat;

extern "C" {
    fn Instant_now() -> f64;
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct Instant(OrderedFloat<f64>);

impl Instant {
    pub fn now() -> Self {
        Instant(OrderedFloat(unsafe { Instant_now() }))
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Duration {
        let total_nanos = ((self.0).0 - (rhs.0).0) * 1_000_000_000.0;
        let total_nanos = total_nanos as u128;
        let secs = total_nanos / 1_000_000_000;
        let nanos = total_nanos % 1_000_000_000;
        Duration::new(secs as _, nanos as _)
    }
}

pub use std::time::Duration;
