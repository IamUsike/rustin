use std::time::{Duration, Instant};

pub trait Limiter {
    fn try_acquire(&mut self, now: Instant) -> bool;
}

pub struct TokenBucket {
    capacity: u32,
    refill_per_sec: f64,
    cur_capacity: f64,
    last_used: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_per_sec: f64, now: Instant) -> Self {
        Self {
            capacity,
            cur_capacity: capacity as f64,
            refill_per_sec,
            last_used: now,
        }
    }
}

impl Limiter for TokenBucket {
    //refilling the bucket and then validate
    //cos shi is done lazily. whenever a req comes
    //and the bucket is empty(but refilled_cap > 0), it should
    //first be filled by the time elapsed and then validated.
    fn try_acquire(&mut self, now: Instant) -> bool {
        //cant directly multiply Instant with f64 for multiply by 1 sec and then add it
        let refilled_cap =
            (now - self.last_used) + Duration::from_secs(1).mul_f64(self.refill_per_sec);

        //refill lazily. ie: just read the below code bro
        // self.cur_capacity = self.capacity.min(refilled_cap) as f64;
        if Duration::from_secs(self.capacity as u64) < refilled_cap {
            self.cur_capacity = self.capacity as f64;
        } else {
            self.cur_capacity = refilled_cap.as_secs_f64();
        }

        if self.cur_capacity > 0.0 {
            self.cur_capacity -= 1.0;
            return true;
        }

        false
    }
}

pub struct SlidingWindowCounter {
    limit: u32,
    window: Duration,
    pre_win_reqs: u32,
    cur_win_reqs: u32,
    cur_win_start: Instant,
    prev_win_end: Instant,
}

impl SlidingWindowCounter {
    pub fn new(limit: u32, window: Duration, now: Instant) -> Self {
        Self {
            limit,
            window,
            cur_win_start: now,
            prev_win_end: now, //doesnt matter even if now -1 cos req = 0
            cur_win_reqs: 0,
            pre_win_reqs: 0,
        }
    }
}

impl Limiter for SlidingWindowCounter {
    fn try_acquire(&mut self, now: Instant) -> bool {
        //check the the window has expired
        if now > self.cur_win_start + self.window {
            self.prev_win_end = self.cur_win_start + self.window;
            self.pre_win_reqs = self.cur_win_reqs;
            self.cur_win_reqs = 0;
            self.cur_win_start = now;
        }

        if self.cur_win_reqs + 1 > self.limit {
            return false;
        }

        //no intermittent traffic ie: win A has traffic, win B => NO traffic, win C => traffic
        //wipe out the prev window in this case
        if self.prev_win_end + self.window > self.cur_win_start {
            self.pre_win_reqs = 0;
        }

        //time elaped in cw
        let t0 = now - self.cur_win_start;
        let fraction_time = t0.as_secs_f64() / self.window.as_secs_f64();
        //no. of reqs allowed in the cur. sliding window
        let estimate = (1.0 - fraction_time) * self.pre_win_reqs as f64 + self.cur_win_reqs as f64;

        if self.cur_win_reqs as f64 + 1.0 > estimate {
            return false;
        }

        self.cur_win_reqs += 1;

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn sec(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    // Calls try_acquire `tries` times at the same instant, returns how many were allowed.
    fn count_allowed(l: &mut impl Limiter, now: Instant, tries: u32) -> u32 {
        (0..tries).filter(|_| l.try_acquire(now)).count() as u32
    }

    // ---------- TokenBucket ----------

    #[test]
    fn bucket_starts_full_and_rejects_when_empty() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);
        assert_eq!(count_allowed(&mut b, t0, 10), 5);
    }

    #[test]
    fn bucket_refills_over_time() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);
        count_allowed(&mut b, t0, 5);
        assert_eq!(count_allowed(&mut b, t0 + sec(1), 10), 1);
        assert_eq!(count_allowed(&mut b, t0 + sec(3), 10), 2);
    }

    #[test]
    fn bucket_keeps_partial_tokens() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);
        count_allowed(&mut b, t0, 5);
        assert!(!b.try_acquire(t0 + ms(500))); // half a token: not enough
        assert!(b.try_acquire(t0 + ms(1000))); // that half must not be thrown away
    }

    #[test]
    fn bucket_never_exceeds_capacity() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);
        assert_eq!(count_allowed(&mut b, t0 + sec(3600), 100), 5);
    }

    #[test]
    fn bucket_fractional_refill_rate() {
        let t0 = Instant::now();
        let mut b = TokenBucket::new(2, 0.5, t0); // 1 token every 2 seconds
        count_allowed(&mut b, t0, 2);
        assert_eq!(count_allowed(&mut b, t0 + sec(1), 5), 0);
        assert_eq!(count_allowed(&mut b, t0 + sec(2), 5), 1);
    }

    // ---------- SlidingWindowCounter ----------

    #[test]
    fn window_allows_limit_then_rejects() {
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        assert_eq!(count_allowed(&mut w, t0, 20), 10);
    }

    #[test]
    fn window_start_of_next_window_still_full() {
        // at exactly t=60s the previous window has weight 1.0
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0, 10);
        assert_eq!(count_allowed(&mut w, t0 + sec(60), 10), 0);
    }

    #[test]
    fn window_weights_previous_window() {
        // halfway into window 1: estimate = cur + 10 * 0.5 = cur + 5
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0, 10);
        assert_eq!(count_allowed(&mut w, t0 + sec(90), 20), 5);
    }

    #[test]
    fn window_weight_uses_position_in_current_window() {
        // 15s into window 1: weight 0.75, estimate = cur + 7.5, so cur 0, 1, 2 pass
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0 + sec(10), 5);
        count_allowed(&mut w, t0 + sec(50), 5);
        assert_eq!(count_allowed(&mut w, t0 + sec(75), 20), 3);
    }

    #[test]
    fn window_rejected_requests_do_not_count() {
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        assert_eq!(count_allowed(&mut w, t0, 15), 10); // 5 rejected
        assert_eq!(count_allowed(&mut w, t0 + sec(90), 20), 5); // prev is 10, not 15
    }

    #[test]
    fn window_forgets_traffic_two_windows_old() {
        // at t=125s, current window is [120, 180), previous [60, 120) was empty
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0, 10);
        assert_eq!(count_allowed(&mut w, t0 + sec(125), 20), 10);
    }

    #[test]
    fn window_skips_idle_windows_without_drift() {
        // idle for many windows, then traffic: windows must stay aligned to t0
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        count_allowed(&mut w, t0 + sec(600), 10); // window [600, 660)
        assert_eq!(count_allowed(&mut w, t0 + sec(690), 20), 5); // halfway into [660, 720)
    }
}
