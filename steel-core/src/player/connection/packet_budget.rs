//! Vanilla `PacketUtils`-style flood control for a single connection.
//!
//! Vanilla guards every inbound packet with three independent allowances
//! (`ServerGamePacketListenerImpl`):
//!
//! * `receivedPackets` — a second-scale bucket sized by `server.properties`
//!   `rate-limit` (`0`, vanilla's default, means unlimited); going over it is an
//!   immediate `disconnect(EXCEEDED_PACKET_RATE)` with the `Packet rate limit
//!   exceeded` kick reason. Steel keeps that setting but always applies the
//!   [`MAX_PACKETS_PER_SECOND`] ceiling, so an unconfigured server is still
//!   bounded.
//! * `packetSizeLimiter` — a shared 1 MiB/s budget across *all* connections for
//!   payload bytes (`BandwidthDebugMonitor`).
//! * `droppedPacketsCounter`/`averagePendingPacketSize` — a leaky bucket on the
//!   pending-queue size that kicks with `EXCEEDED_PACKET_RATE` once a client
//!   queues more than [`DROP_THRESHOLD`] packets it never lets drain.
//!
//! Steel reads packets on a per-connection task, so the per-connection bucket
//! and the payload-byte budget are what actually bound a hostile client. They
//! are also the layer that stops "1,000,000 book packets" style floods before a
//! single one reaches the game loop: a connection that cannot keep its rate can
//! only ever queue [`MAX_PACKETS_PER_SECOND`] packets per second, and repeated
//! protocol violations close it outright (see [`ViolationBudget`]).

use std::time::{Duration, Instant};

/// Hard ceiling on inbound packets per second for one connection.
///
/// Vanilla keeps `rate-limit=0` (unlimited) by default and only enforces a
/// per-second allowance when the operator sets `server.properties` `rate-limit`.
/// Steel keeps the operator setting as the configured limit and adds this
/// ceiling on top, so an unconfigured server is still safe from a single
/// client that tries to flood the game loop. Vanilla's own documented guidance
/// for a public server is `500`, and a legitimate client never exceeds roughly
/// `300`, so the ceiling stays out of the way of normal traffic.
pub const MAX_PACKETS_PER_SECOND: u32 = 500;

/// Vanilla `ServerGamePacketListenerImpl.BANDWIDTH_DEBUG_MONITOR` byte budget:
/// 1 MiB of payload per second shared by every connection.
pub const MAX_PAYLOAD_BYTES_PER_SECOND: u64 = 1 << 20;

/// Payload bytes one connection may borrow from the shared budget per second.
///
/// Vanilla shares a single budget process-wide, which makes one client able to
/// starve the rest. Steel gives each connection a fair share of
/// [`MAX_PAYLOAD_BYTES_PER_SECOND`] instead, which is the same bound for the
/// single-client case without the cross-player denial of service.
pub const MAX_CONNECTION_BYTES_PER_SECOND: u64 = MAX_PAYLOAD_BYTES_PER_SECOND / 8;

/// Vanilla `ServerGamePacketListenerImpl.DROP_THRESHOLD`: queued packets after
/// which the leaky bucket starts dropping and eventually kicks.
pub const DROP_THRESHOLD: u32 = 8;

/// Buckets start empty and refill continuously at the configured rate.
#[derive(Debug, Clone)]
pub struct RateBucket {
    capacity: u64,
    tokens: u64,
    /// Tokens added per second.
    refill_per_second: u64,
    last_refill: Instant,
}

impl RateBucket {
    /// Creates a full bucket holding `capacity` units that refills at
    /// `refill_per_second`.
    #[must_use]
    pub fn new(capacity: u64, refill_per_second: u64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_per_second,
            last_refill: Instant::now(),
        }
    }

    /// Refills for the elapsed time and tries to spend `amount` units.
    ///
    /// Returns `false` when the bucket cannot cover the request, which leaves it
    /// untouched so a caller can retry after the next refill.
    pub fn try_consume(&mut self, amount: u64) -> bool {
        self.refill();
        if self.tokens < amount {
            return false;
        }
        self.tokens -= amount;
        true
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_refill);
        if elapsed.is_zero() {
            return;
        }
        self.last_refill = now;
        // Scale by whole milliseconds so the bucket does not creep ahead for
        // sub-millisecond gaps, and saturate instead of overflowing.
        let millis = elapsed.as_millis() as u64;
        if millis == 0 {
            return;
        }
        let gained = self
            .refill_per_second
            .saturating_mul(millis)
            .checked_div(1000)
            .unwrap_or(0);
        self.tokens = self.tokens.saturating_add(gained).min(self.capacity);
    }
}

impl Default for RateBucket {
    fn default() -> Self {
        Self::new(
            u64::from(MAX_PACKETS_PER_SECOND),
            u64::from(MAX_PACKETS_PER_SECOND),
        )
    }
}

/// Per-connection inbound accounting.
///
/// Not `Clone`: the state is deliberately owned by one connection task.
#[derive(Debug)]
pub struct PacketBudget {
    packets: RateBucket,
    bytes: RateBucket,
    /// Packets that arrived but could not be admitted, and were therefore
    /// dropped rather than acted on. Mirrors vanilla's `droppedPacketsCounter`.
    dropped: u32,
}

impl PacketBudget {
    /// A budget using the hard per-connection ceiling.
    #[must_use]
    pub fn new() -> Self {
        Self::with_packet_limit(MAX_PACKETS_PER_SECOND)
    }

    /// A budget honouring the operator's `rate-limit` setting.
    ///
    /// A non-positive `configured` value means "unlimited" (vanilla's default),
    /// which Steel maps to [`MAX_PACKETS_PER_SECOND`]. A positive value tightens
    /// the allowance further but can never exceed that ceiling.
    #[must_use]
    pub fn with_packet_limit(configured: i32) -> Self {
        let per_second = match u32::try_from(configured) {
            Ok(limit) if limit > 0 => limit.min(MAX_PACKETS_PER_SECOND),
            _ => MAX_PACKETS_PER_SECOND,
        };
        Self {
            packets: RateBucket::new(u64::from(per_second), u64::from(per_second)),
            bytes: RateBucket::new(
                MAX_CONNECTION_BYTES_PER_SECOND,
                MAX_CONNECTION_BYTES_PER_SECOND,
            ),
            dropped: 0,
        }
    }

    /// Admits one inbound packet of `payload_bytes` bytes.
    ///
    /// An empty packet is always admitted so keep-alives cannot be starved by a
    /// full byte bucket. A refusal is recorded as a drop; enough drops in a row
    /// make the caller close the connection.
    pub fn admit(&mut self, payload_bytes: usize) -> Admission {
        let bytes = u64::try_from(payload_bytes).unwrap_or(u64::MAX);
        if bytes > 0 && !self.bytes.try_consume(bytes) {
            return self.reject(RejectionCause::PayloadBytes);
        }
        if !self.packets.try_consume(1) {
            return self.reject(RejectionCause::PacketRate);
        }
        self.dropped = 0;
        Admission::Allowed
    }

    fn reject(&mut self, cause: RejectionCause) -> Admission {
        self.dropped = self.dropped.saturating_add(1);
        Admission::Dropped {
            cause,
            dropped: self.dropped,
        }
    }
}

impl Default for PacketBudget {
    fn default() -> Self {
        Self::new()
    }
}

/// Why a packet was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionCause {
    /// The per-second packet allowance is exhausted.
    PacketRate,
    /// The per-second payload byte allowance is exhausted.
    PayloadBytes,
}

/// Outcome of [`PacketBudget::admit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// The packet may be processed.
    Allowed,
    /// The packet must be discarded. `dropped` is the running count of
    /// consecutive drops.
    Dropped {
        /// The allowance that ran out.
        cause: RejectionCause,
        /// Consecutive drops so far.
        dropped: u32,
    },
}

impl Admission {
    /// Returns whether the packet may be processed.
    #[must_use]
    pub const fn is_allowed(self) -> bool {
        matches!(self, Self::Allowed)
    }

    /// Returns whether enough consecutive drops have accumulated to close the
    /// connection, mirroring vanilla's `EXCEEDED_PACKET_RATE` disconnect.
    #[must_use]
    pub const fn exceeds_drop_threshold(self) -> bool {
        match self {
            Self::Allowed => false,
            Self::Dropped { dropped, .. } => dropped > DROP_THRESHOLD,
        }
    }
}

/// Counts protocol violations for one connection.
///
/// Vanilla raises a `DecoderException` out of the pipeline and closes the
/// channel, so a malformed packet is fatal immediately. Steel additionally
/// tolerates a small number of transient decode failures (a client resizing the
/// window mid-frame, for example) but must never keep serving a peer that keeps
/// sending garbage: past [`MAX_TOLERATED_VIOLATIONS`] the connection is closed.
#[derive(Debug, Default)]
pub struct ViolationBudget {
    tolerated: u32,
}

/// How many malformed packets a connection may send before being closed.
pub const MAX_TOLERATED_VIOLATIONS: u32 = 8;

impl ViolationBudget {
    /// Records one malformed packet; returns whether the connection must close.
    pub fn record(&mut self) -> bool {
        self.tolerated = self.tolerated.saturating_add(1);
        self.exceeds_limit()
    }

    /// Returns whether the tolerated violations are exhausted.
    #[must_use]
    pub const fn exceeds_limit(&self) -> bool {
        self.tolerated > MAX_TOLERATED_VIOLATIONS
    }

    /// Returns the violations recorded so far.
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.tolerated
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        Admission, DROP_THRESHOLD, MAX_CONNECTION_BYTES_PER_SECOND, MAX_PACKETS_PER_SECOND,
        MAX_TOLERATED_VIOLATIONS, PacketBudget, RateBucket, RejectionCause, ViolationBudget,
    };

    #[test]
    fn a_fresh_bucket_spends_its_capacity() {
        let mut bucket = RateBucket::new(4, 4);
        for _ in 0..4 {
            assert!(bucket.try_consume(1));
        }
        assert!(!bucket.try_consume(1), "capacity is exhausted");
    }

    #[test]
    fn a_bucket_refills_over_time() {
        let mut bucket = RateBucket::new(2, 1000);
        assert!(bucket.try_consume(2));
        assert!(!bucket.try_consume(1));
        std::thread::sleep(Duration::from_millis(5));
        assert!(bucket.try_consume(1), "5ms at 1000/s yields tokens");
    }

    #[test]
    fn a_bucket_saturates_at_its_capacity() {
        let mut bucket = RateBucket::new(2, 1_000_000);
        std::thread::sleep(Duration::from_millis(3));
        // A long idle period must not let one packet burst past capacity.
        assert!(bucket.try_consume(2));
        assert!(!bucket.try_consume(1));
    }

    #[test]
    fn a_packet_flood_is_cut_off_after_one_second_of_allowance() {
        let mut budget = PacketBudget::new();
        let mut allowed = 0;
        for _ in 0..=MAX_PACKETS_PER_SECOND {
            if budget.admit(16).is_allowed() {
                allowed += 1;
            }
        }
        assert_eq!(allowed, MAX_PACKETS_PER_SECOND);
    }

    #[test]
    fn exceeding_the_packet_rate_closes_the_connection() {
        let mut budget = PacketBudget::new();
        for _ in 0..MAX_PACKETS_PER_SECOND {
            assert!(budget.admit(1).is_allowed());
        }
        let mut last = budget.admit(1);
        for _ in 0..DROP_THRESHOLD {
            last = budget.admit(1);
        }
        assert!(!last.is_allowed());
        assert!(last.exceeds_drop_threshold(), "a sustained flood must kick");
    }

    #[test]
    fn an_occasional_drop_does_not_kick() {
        let mut budget = PacketBudget::new();
        for _ in 0..MAX_PACKETS_PER_SECOND {
            let _ = budget.admit(1);
        }
        let dropped = budget.admit(1);
        assert_eq!(
            dropped,
            Admission::Dropped {
                cause: RejectionCause::PacketRate,
                dropped: 1
            }
        );
        assert!(!dropped.exceeds_drop_threshold());
    }

    #[test]
    fn a_payload_flood_is_cut_off() {
        let mut budget = PacketBudget::new();
        // The byte bucket holds exactly one second of per-connection payload, so
        // the first full-size frame fits and the second one cannot.
        let frame = MAX_CONNECTION_BYTES_PER_SECOND as usize;
        assert!(budget.admit(frame).is_allowed());
        assert!(!budget.admit(frame).is_allowed());
    }

    #[test]
    fn keep_alives_are_never_refused_for_bytes() {
        let mut budget = PacketBudget::new();
        let frame = MAX_CONNECTION_BYTES_PER_SECOND as usize;
        assert!(budget.admit(frame).is_allowed());
        assert!(budget.admit(0).is_allowed(), "empty packets stay admitted");
    }

    #[test]
    fn a_configured_rate_limit_tightens_the_allowance() {
        let mut budget = PacketBudget::with_packet_limit(4);
        for _ in 0..4 {
            assert!(budget.admit(1).is_allowed());
        }
        assert!(!budget.admit(1).is_allowed(), "the operator limit applies");
    }

    #[test]
    fn a_disabled_rate_limit_falls_back_to_the_ceiling() {
        // Vanilla default `rate-limit=0`, and a nonsensical negative value, must
        // both end up on the hard ceiling rather than disabling protection.
        for configured in [0, -1, i32::MIN] {
            let mut budget = PacketBudget::with_packet_limit(configured);
            for _ in 0..MAX_PACKETS_PER_SECOND {
                assert!(budget.admit(1).is_allowed(), "{configured} keeps the ceiling");
            }
            assert!(!budget.admit(1).is_allowed(), "{configured} does not disable");
        }
    }

    #[test]
    fn a_huge_rate_limit_cannot_exceed_the_ceiling() {
        let mut budget = PacketBudget::with_packet_limit(i32::MAX);
        for _ in 0..MAX_PACKETS_PER_SECOND {
            assert!(budget.admit(1).is_allowed());
        }
        assert!(!budget.admit(1).is_allowed(), "the ceiling always wins");
    }

    #[test]
    fn violations_close_the_connection_after_the_tolerated_count() {
        let mut violations = ViolationBudget::default();
        for _ in 0..MAX_TOLERATED_VIOLATIONS {
            assert!(!violations.record(), "tolerated violations must not close");
        }
        assert!(violations.record(), "one violation past the limit must close");
        assert_eq!(violations.count(), MAX_TOLERATED_VIOLATIONS + 1);
    }
}
