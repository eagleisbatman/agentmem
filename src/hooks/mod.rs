pub mod manager;
pub mod service;
pub mod templates;

pub use service::{install_hooks, list_hooks, test_hook, HookInfo};
