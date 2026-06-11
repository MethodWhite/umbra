#[derive(Clone)]
pub struct SkillManager;

impl SkillManager {
    pub fn new() -> Self {
        Self
    }

    pub fn discover(&self) -> Vec<String> {
        Vec::new()
    }
}
