use anyhow::{Result, anyhow};
use std::collections::HashMap;
use chrono::Utc;

use super::types::{Signal, Order, OrderStatus};

pub struct OrderExecutor {
    max_positions: u32,
    max_daily_loss_percent: f64,
    active_orders: HashMap<String, Order>,
    daily_loss: f64,
    account_balance: f64,
}

impl OrderExecutor {
    pub fn new() -> Self {
        Self {
            max_positions: 5,
            max_daily_loss_percent: 5.0,
            active_orders: HashMap::new(),
            daily_loss: 0.0,
            account_balance: 10000.0,
        }
    }

    pub fn check_risk(&self, signal: &Signal) -> Result<bool> {
        if self.active_orders.len() >= self.max_positions as usize {
            return Err(anyhow!("Máximo de posiciones alcanzado: {}", self.max_positions));
        }
        let risk_amount = signal.volume * signal.confidence;
        let max_loss = self.account_balance * (self.max_daily_loss_percent / 100.0);
        if self.daily_loss + risk_amount > max_loss {
            return Err(anyhow!("Límite de pérdida diaria alcanzado"));
        }
        Ok(true)
    }

    pub fn place_order(&self, signal: Signal) -> Result<Order> {
        let price = self.simulate_price(&signal.symbol);
        Ok(Order {
            order_id: Some(Utc::now().timestamp()),
            signal_id: signal.signal_id,
            timestamp: Utc::now().timestamp(),
            symbol: signal.symbol,
            action: signal.action,
            volume: signal.volume,
            price: Some(price),
            stop_loss: signal.stop_loss,
            take_profit: signal.take_profit,
            status: OrderStatus::Executed,
            error: None,
        })
    }

    pub fn confirm_order(&self, _order: Order) {
        tracing::info!("[Executor] Orden confirmada en MT4");
    }

    fn simulate_price(&self, symbol: &str) -> f64 {
        match symbol {
            "EURUSD" => 1.0850,
            "GBPUSD" => 1.2650,
            "XAUUSD" => 2345.0,
            "BTCUSD" => 68500.0,
            _ => 1.0,
        }
    }
}
