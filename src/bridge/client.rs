use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::types::{Action, Order};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub balance: f64,
    pub equity: f64,
    pub margin: f64,
    pub margin_free: f64,
    pub leverage: u32,
    pub server: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub ticket: i64,
    pub symbol: String,
    pub action: Action,
    pub volume: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub pnl: f64,
    pub swap: f64,
}

pub struct Mt5Client;

impl Mt5Client {
    pub fn get_account_info() -> Result<AccountInfo> {
        let json = super::ffi::call_mt5("get_account_info", "")?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn get_positions() -> Result<Vec<Position>> {
        let json = super::ffi::call_mt5("get_positions", "")?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn place_order(order: Order) -> Result<String> {
        let payload = serde_json::to_string(&order)?;
        let json = super::ffi::call_mt5("place_order", &payload)?;
        Ok(json.trim_matches('"').to_string())
    }

    pub fn get_market_data(symbol: &str) -> Result<MarketData> {
        let json = super::ffi::call_mt5("get_market_data", symbol)?;
        Ok(serde_json::from_str(&json)?)
    }
}
