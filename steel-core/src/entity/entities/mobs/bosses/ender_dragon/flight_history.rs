//! Vanilla `DragonFlightHistory`: the ring buffer of the dragon's recent height
//! and yaw.
//!
//! The dragon samples its own height and yaw once per tick and reads the samples
//! back with a delay to shape its body: the tails lag twelve to sixteen ticks
//! behind, the head pitch comes from the five- versus zero-tick delta, and the wing
//! tilt comes from the five- versus ten-tick delta. Vanilla's buffer is a fixed
//! 64-entry array with a wrapping head index, which this mirrors.
//!
//! Vanilla also offers a partial-tick read for client rendering; Steel only needs
//! the server-side reads, so that overload is omitted.

/// Vanilla `DragonFlightHistory.LENGTH`.
pub const LENGTH: usize = 64;

/// Vanilla `DragonFlightHistory.MASK` (`LENGTH - 1`).
const MASK: usize = LENGTH - 1;

/// One recorded `(y, yRot)` sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlightSample {
    /// The dragon's `y` coordinate when the sample was taken.
    pub y: f64,
    /// The dragon's yaw in degrees when the sample was taken.
    pub y_rot: f32,
}

impl FlightSample {
    /// The zeroed sample vanilla's constructor fills the buffer with.
    #[must_use]
    pub const fn zeroed() -> Self {
        Self { y: 0.0, y_rot: 0.0 }
    }
}

/// Vanilla `DragonFlightHistory`.
#[derive(Debug, Clone)]
pub struct DragonFlightHistory {
    samples: [FlightSample; LENGTH],
    /// `-1` until the first sample is recorded, mirroring vanilla's `head`.
    head: isize,
}

impl Default for DragonFlightHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl DragonFlightHistory {
    /// Creates a history filled with zeroed samples and no head yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            samples: [FlightSample::zeroed(); LENGTH],
            head: -1,
        }
    }

    /// Vanilla `DragonFlightHistory.copyFrom`.
    pub const fn copy_from(&mut self, other: &Self) {
        self.samples = other.samples;
        self.head = other.head;
    }

    /// Vanilla `DragonFlightHistory.record`. The first sample fills the whole
    /// buffer so early reads never observe the initial zeros.
    pub const fn record(&mut self, y: f64, y_rot: f32) {
        let sample = FlightSample { y, y_rot };
        if self.head < 0 {
            self.samples = [sample; LENGTH];
        }
        self.head = (self.head + 1) % LENGTH as isize;
        self.samples[self.head as usize] = sample;
    }

    /// Vanilla `DragonFlightHistory.get(int delay)`.
    ///
    /// A read before the first [`Self::record`] returns the zeroed sample, which is
    /// what vanilla's constructor leaves in the buffer.
    #[must_use]
    pub const fn get(&self, delay: usize) -> FlightSample {
        if self.head < 0 {
            return FlightSample::zeroed();
        }
        // Vanilla computes `(head - delay) & MASK` in two's complement; wrapping
        // subtraction then masking is the same ring arithmetic for a non-negative
        // head.
        let index = (self.head as usize).wrapping_sub(delay) & MASK;
        self.samples[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_history_reports_zeroed_samples() {
        let history = DragonFlightHistory::new();
        assert_eq!(history.get(0), FlightSample::zeroed());
        assert_eq!(history.get(5).y, 0.0);
    }

    #[test]
    fn the_first_record_fills_the_whole_buffer() {
        let mut history = DragonFlightHistory::new();
        history.record(64.0, 90.0);

        for delay in 0..LENGTH {
            let sample = history.get(delay);
            assert_eq!(
                sample.y, 64.0,
                "vanilla fills every slot on the first record"
            );
            assert_eq!(sample.y_rot, 90.0);
        }
    }

    #[test]
    fn samples_are_read_back_by_delay_and_wrap_around() {
        let mut history = DragonFlightHistory::new();
        for tick in 0..(LENGTH + 3) {
            history.record(f64::from(tick as i32), 0.0);
        }

        assert_eq!(history.get(0).y, (LENGTH + 2) as f64);
        assert_eq!(history.get(5).y, (LENGTH - 3) as f64);
        assert_eq!(
            history.get(LENGTH).y,
            (LENGTH + 2) as f64,
            "a full-buffer delay lands on the same slot as no delay"
        );
    }
}
