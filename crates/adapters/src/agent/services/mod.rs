pub mod nutrition;

pub trait AgentServiceProgress: Send + Sync {
    fn status(&self, label: &'static str);
}
