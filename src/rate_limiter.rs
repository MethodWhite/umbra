// Zone 6 — Research/Stubs (server-gated)
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;

pub struct ActionRateLimiter {
    window: Mutex<VecDeque<Instant>>,
    max_actions: u32,
    window_secs: u64,
}

impl ActionRateLimiter {
    pub fn new() -> Self {
        Self {
            window: Mutex::new(VecDeque::new()),
            max_actions: 30,
            window_secs: 60,
        }
    }

    pub async fn check_action(&self, _action: &str) -> bool {
        let mut window = self.window.lock().unwrap();
        let now = Instant::now();
        let cutoff = now - std::time::Duration::from_secs(self.window_secs);

        while let Some(&time) = window.front() {
            if time < cutoff {
                window.pop_front();
            } else {
                break;
            }
        }

        if window.len() < self.max_actions as usize {
            window.push_back(now);
            true
        } else {
            false
        }
    }
}
