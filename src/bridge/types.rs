// Zone 5 — Bridge/External
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub signal_id: String,
    pub timestamp: i64,
    pub symbol: String,
    pub action: Action,
    pub volume: f64,
    pub confidence: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Buy,
    Sell,
    Close,
    Hold,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Buy => write!(f, "BUY"),
            Action::Sell => write!(f, "SELL"),
            Action::Close => write!(f, "CLOSE"),
            Action::Hold => write!(f, "HOLD"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: Option<i64>,
    pub signal_id: String,
    pub timestamp: i64,
    pub symbol: String,
    pub action: Action,
    pub volume: f64,
    pub price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub status: OrderStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Executed,
    Rejected,
    Cancelled,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "PENDING"),
            OrderStatus::Executed => write!(f, "EXECUTED"),
            OrderStatus::Rejected => write!(f, "REJECTED"),
            OrderStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiConfig {
    pub mode: TradingMode,
    pub max_risk_percent: f64,
    pub max_positions: u32,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingMode {
    Paper,
    Live,
}

impl std::fmt::Display for TradingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradingMode::Paper => write!(f, "paper"),
            TradingMode::Live => write!(f, "live"),
        }
    }
}

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
