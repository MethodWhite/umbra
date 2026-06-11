use crate::domain::ports::execution_port::{ExecutionPort, ExecutionResult};

pub struct ExecuteCommandUseCase;

impl ExecuteCommandUseCase {
    pub async fn execute(&self, _command: &str, _executor: &dyn ExecutionPort) -> Result<ExecutionResult, String> {
        todo!("Implement command execution")
    }
}
