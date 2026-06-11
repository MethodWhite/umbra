use crate::domain::models::trading::Order;

pub enum ExecutionResult {
    Success(String),
    Failure(String),
    Pending(String),
}

pub trait ExecutionPort: Send + Sync {
    fn execute_order(&self, order: &Order) -> Result<ExecutionResult, String>;
    fn cancel_order(&self, ticket: u64) -> Result<ExecutionResult, String>;
    fn get_positions(&self) -> Result<Vec<crate::domain::models::trading::Position>, String>;
    fn get_account_info(&self) -> Result<crate::domain::models::trading::AccountInfo, String>;
}
