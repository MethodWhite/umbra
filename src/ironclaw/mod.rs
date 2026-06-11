// Zone 6 — Research/Stubs (server-gated)
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub use crate::rate_limiter::ActionRateLimiter;

const MAX_INPUT_LENGTH: usize = 32_000;
const MAX_OUTPUT_LENGTH: usize = 8_000;

pub struct IronClaw {
    rate_limiter: ActionRateLimiter,
    constraints: Constraints,
    stats: Arc<ClawStats>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct Constraints {
    max_positions: u32,
    max_daily_loss_pct: f64,
    max_conversation_turns: u32,
    max_input_len: usize,
    max_output_len: usize,
    max_iterations: u32,
    blocked_actions: Vec<String>,
}

struct ClawStats {
    total_actions: AtomicU64,
    blocked_actions: AtomicU64,
    total_tokens_in: AtomicU64,
    total_tokens_out: AtomicU64,
}

impl IronClaw {
    pub fn new() -> Self {
        Self {
            rate_limiter: ActionRateLimiter::new(),
            constraints: Constraints {
                max_positions: 5,
                max_daily_loss_pct: 5.0,
                max_conversation_turns: 200,
                max_input_len: MAX_INPUT_LENGTH,
                max_output_len: MAX_OUTPUT_LENGTH,
                max_iterations: 12,
        blocked_actions: vec![
            "/bin/rm".into(), "/usr/bin/rm".into(), "sudo".into(), "shutdown".into(), "reboot".into(),
            "dd".into(), "mkfs".into(), "format".into(), ">".into(),
            "rmdir".into(), "unlink".into(), "fdisk".into(), "parted".into(),
            "pvcreate".into(), "vgcreate".into(), "lvcreate".into(),
            "cryptsetup".into(), "flashrom".into(), "fastboot".into(), "heimdall".into(),
        ],
            },
            stats: Arc::new(ClawStats {
                total_actions: AtomicU64::new(0),
                blocked_actions: AtomicU64::new(0),
                total_tokens_in: AtomicU64::new(0),
                total_tokens_out: AtomicU64::new(0),
            }),
        }
    }

    pub async fn validate_action(&self, action: &str, _args: &[&str]) -> Result<(), ClawBlock> {
        self.stats.total_actions.fetch_add(1, Ordering::Relaxed);

        if !self.rate_limiter.check_action(action).await {
            self.stats.blocked_actions.fetch_add(1, Ordering::Relaxed);
            return Err(ClawBlock::RateLimit("Rate limit exceeded: maximum 30 actions per minute".into()));
        }

        if self.constraints.blocked_actions.iter().any(|b| {
            let pattern = b.as_str();
            action == pattern
                || action.starts_with(&format!("{} ", pattern))
                || action.contains(&format!(" {} ", pattern))
                || action.ends_with(&format!(" {}", pattern))
        }) {
            self.stats.blocked_actions.fetch_add(1, Ordering::Relaxed);
            return Err(ClawBlock::ActionBlocked(format!("Action '{}' is in the blocklist", action)));
        }

        Ok(())
    }

    pub async fn validate_input(&self, input: &str) -> Result<(), ClawBlock> {
        if input.len() > self.constraints.max_input_len {
            return Err(ClawBlock::InputTooLong(input.len(), self.constraints.max_input_len));
        }
        Ok(())
    }

    pub async fn validate_output(&self, output: &str) -> Result<(), ClawBlock> {
        if output.len() > self.constraints.max_output_len {
            return Err(ClawBlock::OutputTooLong(output.len(), self.constraints.max_output_len));
        }
        Ok(())
    }

    pub fn track_tokens(&self, input_t: u64, output_t: u64) {
        self.stats.total_tokens_in.fetch_add(input_t, Ordering::Relaxed);
        self.stats.total_tokens_out.fetch_add(output_t, Ordering::Relaxed);
    }

    pub fn stats(&self) -> ClawStatsSnapshot {
        ClawStatsSnapshot {
            total_actions: self.stats.total_actions.load(Ordering::Relaxed),
            blocked_actions: self.stats.blocked_actions.load(Ordering::Relaxed),
            total_tokens_in: self.stats.total_tokens_in.load(Ordering::Relaxed),
            total_tokens_out: self.stats.total_tokens_out.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClawBlock {
    RateLimit(String),
    ActionBlocked(String),
    InputTooLong(usize, usize),
    OutputTooLong(usize, usize),
    ResourceExceeded(String),
}

impl std::fmt::Display for ClawBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClawBlock::RateLimit(msg) => write!(f, "[IronClaw] Rate Limit: {}", msg),
            ClawBlock::ActionBlocked(msg) => write!(f, "[IronClaw] Action Blocked: {}", msg),
            ClawBlock::InputTooLong(actual, max) => write!(f, "[IronClaw] Input too long: {} characters (max {})", actual, max),
            ClawBlock::OutputTooLong(actual, max) => write!(f, "[IronClaw] Output too long: {} characters (max {})", actual, max),
            ClawBlock::ResourceExceeded(msg) => write!(f, "[IronClaw] Resource exceeded: {}", msg),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ClawStatsSnapshot {
    pub total_actions: u64,
    pub blocked_actions: u64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
}
