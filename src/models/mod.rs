use serde::Serialize;

/// Health check response payload.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: &'static str,
}

impl HealthResponse {
    /// Create a healthy response payload.
    #[must_use]
    pub const fn healthy() -> Self {
        Self { status: "OK" }
    }
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self::healthy()
    }
}
