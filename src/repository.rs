#[derive(Clone)]
pub struct AppState;

impl AppState {
    pub fn new(_config: &crate::config::Config) -> Self {
        AppState
    }
}
