// Zone 5 — Bridge/External
pub mod ffi;
pub mod signals;
pub mod executor;
pub mod sandbox;
pub mod client;
pub mod types;

pub use ffi::FfiBridge;
pub use signals::SignalPipeline;
pub use executor::OrderExecutor;
pub use sandbox::StrategySandbox;
pub use types::{Signal, Order, Action, OrderStatus, TradingMode, FfiConfig};
pub use client::{Mt5Client, AccountInfo, MarketData, Position};

use anyhow::Result;

pub struct Mt4Bridge {
    pub signals: SignalPipeline,
    pub executor: OrderExecutor,
    pub sandbox: StrategySandbox,
    pub mode: TradingMode,
}

impl Mt4Bridge {
    pub fn new() -> Self {
        Self {
            signals: SignalPipeline::new(),
            executor: OrderExecutor::new(),
            sandbox: StrategySandbox::new(),
            mode: TradingMode::Paper,
        }
    }

    pub fn with_config(config: FfiConfig) -> Self {
        Self {
            signals: SignalPipeline::new(),
            executor: OrderExecutor::new(),
            sandbox: StrategySandbox::new(),
            mode: config.mode,
        }
    }

    pub fn process_signal(&self, signal: Signal) -> Result<Order> {
        let validated = self.signals.validate(&signal)?;
        let risk_ok = self.executor.check_risk(&validated)?;
        if risk_ok {
            self.executor.place_order(validated)
        } else {
            Ok(Order {
                order_id: None,
                signal_id: signal.signal_id,
                timestamp: chrono::Utc::now().timestamp(),
                symbol: signal.symbol,
                action: Action::Hold,
                volume: 0.0,
                price: None,
                stop_loss: None,
                take_profit: None,
                status: OrderStatus::Rejected,
                error: Some("Risk check failed".into()),
            })
        }
    }
}
