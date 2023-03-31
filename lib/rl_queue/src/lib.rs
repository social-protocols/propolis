use std::time::{Duration, Instant};

/// Simple limiter for a floating point value
pub struct RateLimiter {
    pub period: Duration,
    pub allowed_quota: f64,
    quota: f64,
    last_reset: Instant,
}

#[derive(PartialEq, Debug)]
pub enum QuotaState {
    Remaining(f64),
    ExceededUntil(Instant),
}

impl RateLimiter {
    /// Create a ratelimiter from an allowed quota and a reset period
    pub fn new(allowed_quota: f64, period: Duration) -> Self {
        RateLimiter {
            period,
            allowed_quota,
            quota: 0.0,
            last_reset: Instant::now(),
        }
    }

    /// Returns true if quota is below allowed threshold. Resets quota if over period.
    pub fn check(&mut self) -> bool {
        let now = Instant::now();
        if self.last_reset + self.period < now {
            self.quota = 0.0;
            self.last_reset = now;
        }
        self.quota <= self.allowed_quota
    }

    /// Adds to quota & returns remaining quota or when it is going to be reset
    pub fn add<T: Into<f64>>(&mut self, quota: T) -> QuotaState {
        self.check();
        let quota = quota.into();
        self.quota += quota;
        if self.quota - quota <= self.allowed_quota {
            QuotaState::Remaining(self.allowed_quota - self.quota)
        } else {
            QuotaState::ExceededUntil(Instant::now() + self.period)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{RateLimiter, QuotaState};

    #[test]
    fn test_rate_limiter() {
        let mut rl = RateLimiter::new(100.0, Duration::from_secs(1));

        assert_eq!(rl.check(), true);
        assert_eq!(rl.add(50), QuotaState::Remaining(50.0));
        assert_eq!(rl.add(50), QuotaState::Remaining(0.0));

        match rl.add(1) {
            QuotaState::ExceededUntil(_) => assert!(true),
            _ => assert!(false),
        }
    }
}
