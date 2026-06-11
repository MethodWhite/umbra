use std::collections::VecDeque;
use super::types::Signal;

pub struct BacktestResult {
    pub total_trades: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub profit_loss: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
}

pub struct StrategySandbox {
    history: VecDeque<Signal>,
}

impl StrategySandbox {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(1000),
        }
    }

    pub fn record_signal(&mut self, signal: Signal) {
        if self.history.len() >= 1000 {
            self.history.pop_front();
        }
        self.history.push_back(signal);
    }

    pub fn backtest(&self, signals: &[Signal]) -> BacktestResult {
        let mut wins = 0u32;
        let mut losses = 0u32;
        let mut profit_loss = 0.0f64;
        let mut peak = 0.0f64;
        let mut max_drawdown = 0.0f64;
        let mut returns = Vec::new();

        for signal in signals {
            let result = self.simulate_trade(signal);
            profit_loss += result;
            returns.push(result);

            if result > 0.0 {
                wins += 1;
            } else {
                losses += 1;
            }

            if profit_loss > peak {
                peak = profit_loss;
            }
            let drawdown = if peak > 0.0 { (peak - profit_loss) / peak } else { 0.0 };
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        let total = wins + losses;
        let win_rate = if total > 0 { wins as f64 / total as f64 } else { 0.0 };
        let mean = if !returns.is_empty() { returns.iter().sum::<f64>() / returns.len() as f64 } else { 0.0 };
        let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len().max(1) as f64;
        let sharpe = if variance > 0.0 { mean / variance.sqrt() * (252.0_f64).sqrt() } else { 0.0 };

        BacktestResult {
            total_trades: total,
            wins,
            losses,
            win_rate,
            profit_loss,
            max_drawdown,
            sharpe_ratio: sharpe,
        }
    }

    fn simulate_trade(&self, signal: &Signal) -> f64 {
        let price_change = match signal.symbol.as_str() {
            "EURUSD" => (rand::random::<f64>() - 0.48) * 0.02,
            "XAUUSD" => (rand::random::<f64>() - 0.47) * 0.03,
            _ => (rand::random::<f64>() - 0.49) * 0.015,
        };
        signal.volume * price_change * signal.confidence
    }
}
