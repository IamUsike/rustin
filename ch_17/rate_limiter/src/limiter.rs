use std::time::{Duration, Instant};

pub trait Limiter {
    fn try_acquire(&mut self, now: Instant) -> bool;
}

#[derive(Debug)]
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
        let mut refilled_cap = Duration::from_secs(0);

        //lazy refil karke ka capacity?... hmm
        let elapsed = now.duration_since(self.last_used);
        if !elapsed.is_zero() {
            refilled_cap = (now - self.last_used).mul_f64(self.refill_per_sec);
        }

        //refill lazily. ie: just read the below code bro
        // self.cur_capacity = self.capacity.min(refilled_cap) as f64;
        if Duration::from_secs(self.capacity as u64)
            < refilled_cap + Duration::from_secs_f64(self.cur_capacity)
        {
            self.cur_capacity = self.capacity as f64;
        } else {
            self.cur_capacity += refilled_cap.as_secs_f64();
        }

        self.last_used = now;

        //if cur cap is < 1. just ignore it, no point making it 0 ig
        //i mean it could still be 1 req upar neeche but f it.
        //or else one more if statement and make it 0
        if self.cur_capacity >= 1.0 {
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
        // Check if the window has expired
        if now > self.cur_win_start + self.window {
            self.prev_win_end = self.cur_win_start + self.window;
            self.pre_win_reqs = self.cur_win_reqs;
            self.cur_win_reqs = 0;
            self.cur_win_start = now;
        }

        // no intermittent traffic ie: win A has traffic, win B => NO traffic, win C => traffic
        // Wipe out the prev window in this case
        if self.prev_win_end + self.window < self.cur_win_start {
            self.pre_win_reqs = 0;
        }

        // if e secs have passed in the cur win
        // estimate = (window - e)/window * prev_win_reqs + cur_win_reqs;

        // time elapsed in cw
        let t0 = (now - self.prev_win_end).as_secs_f64();

        // in case the prev window didnt have any req, hmm in hindsight
        // even without this, the code should work fine cos in that case
        // prev win reqs == 0 => estimate ka first half == 0
        // t0 = t0.min(self.window.as_secs_f64());

        let fraction_time = t0 / self.window.as_secs_f64();
        // no. of reqs allowed in the cur. sliding window
        let estimate = (1.0 - fraction_time) * self.pre_win_reqs as f64 + self.cur_win_reqs as f64;

        if estimate >= self.limit as f64 {
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
        dbg!("bucket nigs");
        dbg!(&b);
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

    #[test]
    fn window_late_first_request_does_not_shift_windows() {
        // windows stay [0,60) [60,120) [120,180) no matter when requests arrive
        let t0 = Instant::now();
        let mut w = SlidingWindowCounter::new(10, sec(60), t0);
        assert_eq!(count_allowed(&mut w, t0 + sec(59), 10), 10); // [0,60) = 10
        // t=100: 40s into [60,120), prev weight 1/3, estimate = cur + 3.33
        assert_eq!(count_allowed(&mut w, t0 + sec(100), 20), 7);
        // t=150: 30s into [120,180), prev = 7 (from [60,120)), weight 0.5, estimate = cur + 3.5
        assert_eq!(count_allowed(&mut w, t0 + sec(150), 20), 7);
    }

    #[test]
    fn bucket_out_of_order_timestamps_do_not_mint_tokens() {
        // Stage 2 preview: with many tasks, an older `now` can arrive after a newer one
        let t0 = Instant::now();
        let mut b = TokenBucket::new(5, 1.0, t0);
        count_allowed(&mut b, t0 + sec(10), 5); // drained at t=10
        b.try_acquire(t0 + sec(9)); // late, older timestamp
        assert_eq!(count_allowed(&mut b, t0 + sec(10), 5), 0); // no time passed since t=10
    }
}
