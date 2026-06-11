use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct GhostMonitor {
    heartbeats: VecDeque<Instant>,
    max_samples: usize,
    timeout: Duration,
}

impl GhostMonitor {
    pub fn new() -> Self {
        GhostMonitor {
            heartbeats: VecDeque::with_capacity(100),
            max_samples: 100,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn heartbeat(&mut self) {
        if self.heartbeats.len() >= self.max_samples {
            self.heartbeats.pop_front();
        }
        self.heartbeats.push_back(Instant::now());
    }

    pub fn is_alive(&self) -> bool {
        self.heartbeats
            .back()
            .map_or(false, |t| t.elapsed() < self.timeout)
    }

    pub fn uptime(&self) -> Option<Duration> {
        self.heartbeats.front().map(|t| t.elapsed())
    }

    pub fn heartbeat_count(&self) -> usize {
        self.heartbeats.len()
    }
}
