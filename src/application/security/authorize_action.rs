pub struct AuthorizeActionUseCase;

impl AuthorizeActionUseCase {
    pub async fn execute(&self, _user_id: &str, _action: &str, _resource: &str) -> Result<bool, String> {
        todo!("Implement action authorization")
    }
}
