pub struct AuthorizeActionUseCase;

impl AuthorizeActionUseCase {
    pub async fn execute(&self, user_id: &str, action: &str, resource: &str) -> Result<bool, String> {
        todo!("Implement action authorization")
    }
}
