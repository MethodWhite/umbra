use crate::domain::ports::execution_port::{ExecutionPort, ExecutionResult};

pub struct ExecuteCommandUseCase;

impl ExecuteCommandUseCase {
    pub async fn execute(&self, command: &str, executor: &dyn ExecutionPort) -> Result<ExecutionResult, String> {
        todo!("Implement command execution")
    }
}
