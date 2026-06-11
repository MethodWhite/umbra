use crate::domain::ports::VaultRepository;

pub struct LockUseCase;

impl LockUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, vault: &mut dyn VaultRepository) {
        vault.lock();
    }
}
