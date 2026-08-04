/// Error returned when a frontend data helper fails to talk to the backend.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RequestError {
    /// Failed to serialize the request body.
    #[error("Failed to prepare request data: {0}")]
    Serialize(String),
    /// Failed to build the HTTP request.
    #[error("Could not start the request: {0}")]
    BuildRequest(String),
    /// The network request failed.
    #[error("Network error: {0}")]
    Network(String),
    /// The server returned a non-success status.
    #[error("Server error (HTTP {status}): {body}")]
    Server { status: u16, body: String },
    /// The response body could not be parsed.
    #[error("Failed to read response: {0}")]
    Parse(String),
}

impl RequestError {
    /// Returns a human-readable message suitable for toast or inline error UI.
    pub fn message(&self) -> String {
        self.to_string()
    }
}
